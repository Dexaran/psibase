use libpsibase::*;
use serde::Serialize;
use serde_json::json;

#[psi_macros::contract(example_iface)]
mod example_impl {
    use crate::*;

    #[derive(psi_macros::Fracpack, Serialize)]
    pub struct AccountName {
        num: AccountNum,
        name: String,
    }

    #[action]
    #[allow(unused_variables)]
    pub fn startup(next_account_num: AccountNum, existing_accounts: Vec<AccountName>) {
        write_console("**** startup\n");
        write_console(
            serde_json::to_string(&json!({
                "next_account_num": next_account_num,
                "existing_accounts": existing_accounts,
            }))
            .unwrap()
            .as_str(),
        );
        // abort_message("**** startup\n");
    }

    #[action]
    #[allow(unused_variables)]
    pub fn create_account(name: String, auth_contract: String, allow_sudo: bool) -> AccountNum {
        write_console("**** create_account\n");
        write_console(
            serde_json::to_string(&json!({
                "name": name,
                "authContract": auth_contract,
                "allowSudo": allow_sudo,
            }))
            .unwrap()
            .as_str(),
        );
        abort_message("**** create_account\n");
    }
}
