use anyhow::Context;
use chrono::{Duration, Utc};
use clap::{Parser, Subcommand};
use custom_error::custom_error;
use fracpack::Packable;
use libpsibase::{
    account, get_tapos_for_head, method, push_transaction, sign_transaction, AccountNumber, Action,
    ExactAccountNumber, Fracpack, PrivateKey, PublicKey, Tapos, TimePointSec, Transaction,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};

mod boot;

custom_error! { Error
    OneCreateOnly       = "--create-account and --create-insecure-account cannot be used together",
    Msg{s:String}       = "{s}",
}

/// Interact with a running psinode
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    // TODO: load default from environment variable
    /// API Endpoint
    #[clap(
        short = 'a',
        long,
        value_name = "URL",
        default_value = "http://psibase.127.0.0.1.sslip.io:8080/"
    )]
    api: Url,

    /// Sign with this key (repeatable)
    #[clap(short = 'k', long, value_name = "KEY")]
    sign: Vec<PrivateKey>,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Boot a development chain
    Boot {},

    /// Deploy a contract
    Deploy {
        /// Account to deploy contract on
        account: ExactAccountNumber,

        /// Filename containing the contract
        filename: String,

        /// Create the account if it doesn't exist. Also set the account to
        /// authenticate using this key, even if the account already existed.
        #[clap(short = 'c', long, value_name = "KEY")]
        create_account: Option<PublicKey>,

        /// Create the account if it doesn't exist. The account won't be secured;
        /// anyone can authorize as this account without signing. Caution: this option
        /// should not be used on production or public chains.
        #[clap(short = 'i', long)]
        create_insecure_account: bool,

        /// Register the contract with ProxySys. This allows the contract to host a
        /// website, serve RPC requests, and serve GraphQL requests.
        #[clap(short = 'p', long)]
        register_proxy: bool,
    },

    /// Upload a file to a contract
    Upload {
        /// Contract to upload to
        contract: ExactAccountNumber,

        /// Path to store within contract
        path: String,

        /// MIME content type of file
        content_type: String,

        /// Filename to upload
        filename: String,
    },
}

#[allow(dead_code)] // TODO: move to lib if still needed
fn to_hex(bytes: &[u8]) -> String {
    let mut result: Vec<u8> = Vec::with_capacity(bytes.len() * 2);
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        result.push(DIGITS[(byte >> 4) as usize]);
        result.push(DIGITS[(byte & 0x0f) as usize]);
    }
    String::from_utf8(result).unwrap()
}

#[derive(Serialize, Deserialize, Fracpack)]
pub struct NewAccountAction {
    pub account: AccountNumber,
    pub auth_contract: AccountNumber,
    pub require_new: bool,
}

fn new_account_action(account: AccountNumber) -> Action {
    let new_account_action = NewAccountAction {
        account,
        auth_contract: account!("auth-fake-sys"),
        require_new: false,
    };
    Action {
        sender: account!("account-sys"),
        contract: account!("account-sys"),
        method: method!("newAccount"),
        raw_data: new_account_action.packed_bytes(),
    }
}

fn set_key_action(account: AccountNumber, key: &PublicKey) -> Action {
    Action {
        sender: account,
        contract: account!("auth-ec-sys"),
        method: method!("setKey"),
        raw_data: (key.clone(),).packed_bytes(),
    }
}

fn set_auth_contract_action(account: AccountNumber, auth_contract: AccountNumber) -> Action {
    Action {
        sender: account,
        contract: account!("account-sys"),
        method: method!("setAuthCntr"),
        raw_data: (auth_contract,).packed_bytes(),
    }
}

#[derive(Serialize, Deserialize, Fracpack)]
pub struct SetCodeAction {
    pub contract: AccountNumber,
    pub vm_type: i8,
    pub vm_version: i8,
    pub code: Vec<u8>,
}

fn set_code_action(account: AccountNumber, wasm: Vec<u8>) -> Action {
    let set_code_action = SetCodeAction {
        contract: account,
        vm_type: 0,
        vm_version: 0,
        code: wasm,
    };
    Action {
        sender: account,
        contract: account!("transact-sys"),
        method: method!("setCode"),
        raw_data: set_code_action.packed_bytes(),
    }
}

fn reg_server(contract: AccountNumber, server_contract: AccountNumber) -> Action {
    // todo: should we convert this action data to a proper struct?
    let data = (server_contract,);

    Action {
        sender: contract,
        contract: account!("proxy-sys"),
        method: method!("registerServer"),
        raw_data: data.packed_bytes(),
    }
}

fn store_sys(contract: AccountNumber, path: &str, content_type: &str, content: &[u8]) -> Action {
    let data = (path.to_string(), content_type.to_string(), content.to_vec());
    Action {
        sender: contract,
        contract,
        method: method!("storeSys"),
        raw_data: data.packed_bytes(),
    }
}

pub fn without_tapos(actions: Vec<Action>) -> Transaction {
    let now_plus_10secs = Utc::now() + Duration::seconds(10);
    let expiration = TimePointSec {
        seconds: now_plus_10secs.timestamp() as u32,
    };
    Transaction {
        tapos: Tapos {
            expiration,
            ref_block_suffix: 0,
            flags: 0,
            ref_block_index: 0,
        },
        actions,
        claims: vec![],
    }
}

pub async fn with_tapos(
    base_url: &Url,
    client: reqwest::Client,
    actions: Vec<Action>,
) -> Result<Transaction, anyhow::Error> {
    let tapos = get_tapos_for_head(base_url, client).await?;
    let now_plus_10secs = Utc::now() + Duration::seconds(10);
    let expiration = TimePointSec {
        seconds: now_plus_10secs.timestamp() as u32,
    };
    Ok(Transaction {
        tapos: Tapos {
            expiration,
            ref_block_suffix: tapos.ref_block_suffix,
            flags: 0,
            ref_block_index: tapos.ref_block_index,
        },
        actions,
        claims: vec![],
    })
}

async fn deploy(
    args: &Args,
    client: reqwest::Client,
    account: AccountNumber,
    filename: &str,
    create_account: &Option<PublicKey>,
    create_insecure_account: bool,
    register_proxy: bool,
) -> Result<(), anyhow::Error> {
    let wasm = std::fs::read(filename).with_context(|| format!("Can not read {}", filename))?;

    let mut actions: Vec<Action> = Vec::new();

    if create_account.is_some() && create_insecure_account {
        return Err(Error::OneCreateOnly.into());
    }

    if create_account.is_some() || create_insecure_account {
        actions.push(new_account_action(account));
    }

    // This happens before the set_code as a safety measure.
    // If the set_code was first, and the user didn't actually
    // have the matching private key, then the transaction would
    // lock out the user. Putting this before the set_code causes
    // the set_code to require the private key, failing the transaction
    // if the user doesn't have it.
    if let Some(key) = create_account {
        actions.push(set_key_action(account, key));
        actions.push(set_auth_contract_action(account, account!("auth-ec-sys")));
    }

    actions.push(set_code_action(account, wasm));

    if register_proxy {
        actions.push(reg_server(account, account));
    }

    let trx = with_tapos(&args.api, client.clone(), actions).await?;
    push_transaction(
        &args.api,
        client,
        sign_transaction(trx, &args.sign)?.packed_bytes(),
    )
    .await?;
    println!("Ok");
    Ok(())
}

async fn upload(
    args: &Args,
    client: reqwest::Client,
    contract: AccountNumber,
    path: &str,
    content_type: &str,
    filename: &str,
) -> Result<(), anyhow::Error> {
    let actions = vec![store_sys(
        contract,
        path,
        content_type,
        &std::fs::read(filename).with_context(|| format!("Can not read {}", filename))?,
    )];
    let trx = with_tapos(&args.api, client.clone(), actions).await?;

    push_transaction(
        &args.api,
        client,
        sign_transaction(trx, &args.sign)?.packed_bytes(),
    )
    .await?;
    println!("Ok");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let client = reqwest::Client::new();
    match &args.command {
        Commands::Boot {} => boot::boot(&args, client).await?,
        Commands::Deploy {
            account,
            filename,
            create_account,
            create_insecure_account,
            register_proxy,
        } => {
            deploy(
                &args,
                client,
                (*account).into(),
                filename,
                create_account,
                *create_insecure_account,
                *register_proxy,
            )
            .await?
        }
        Commands::Upload {
            contract,
            path,
            content_type,
            filename,
        } => {
            upload(
                &args,
                client,
                (*contract).into(),
                path,
                content_type,
                filename,
            )
            .await?
        }
    }

    Ok(())
}
