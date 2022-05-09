use clap::{Parser, Subcommand};
use mdbook::preprocess::CmdPreprocessor;
use mdbook::BookItem;
use regex::{CaptureMatches, Captures, Regex};
use std::cell::Cell;
use std::collections::HashMap;
use std::fs::DirEntry;
use std::path::Path;
use std::{fs, io};
use yaml_rust::{Yaml, YamlLoader};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Supports { renderer: String },
}

fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn process_directive<'r, 't>(
    map: &HashMap<&str, &Yaml>,
    global_namespace: &Yaml,
    path: &str,
    matches: CaptureMatches<'r, 't>,
) -> String {
    let mut item = global_namespace;
    let empty = Vec::new();
    for matched_path in matches {
        let items = item["ChildNamespaces"].as_vec().unwrap_or(&empty).iter();
        let items = items.chain(item["ChildRecords"].as_vec().unwrap_or(&empty));
        let items = items.chain(item["ChildFunctions"].as_vec().unwrap_or(&empty));
        let items: Vec<&Yaml> = items.collect();
        let pos = items.iter().position(|item| {
            item["Name"].as_str().unwrap_or("") == matched_path.get(1).unwrap().as_str()
        });
        if let Some(pos) = pos {
            let next_item = map.get(items[pos]["USR"].as_str().unwrap());
            if let Some(next_item) = next_item {
                item = *next_item;
            } else {
                item = items[pos];
            }
        } else {
            panic!("{} not found", path);
        }
    }

    // eprintln!("{:?}", item);
    String::new()
}

fn main() -> Result<(), anyhow::Error> {
    let re1 = Regex::new(r"[{][{] *#cpp-doc +(::[^ ]+)+ *[}][}]").unwrap();
    let re2 = Regex::new(r"::([^: ]+)").unwrap();
    let args = Args::parse();
    if args.command.is_some() {
        return Ok(());
    }

    let (ctx, mut book) = CmdPreprocessor::parse_input(io::stdin())?;
    let mut path = ctx.root;
    path.push("../../build/doc-wasm");
    let definitions = Cell::new(Vec::new());
    visit_dirs(&path, &|entry| {
        eprintln!("{}", entry.path().to_str().unwrap());
        let mut defs = definitions.take();
        defs.append(
            &mut YamlLoader::load_from_str(
                &fs::read_to_string(entry.path().to_str().unwrap()).unwrap(),
            )
            .unwrap(),
        );
        definitions.set(defs);
    })?;
    let definitions = definitions.take();
    let mut map = HashMap::new();
    for x in definitions.iter() {
        map.insert(x["USR"].as_str().unwrap(), x);
    }

    if let Some(global_namespace) = map.get("0000000000000000000000000000000000000000") {
        book.for_each_mut(|item: &mut BookItem| match item {
            BookItem::Chapter(chapter) => {
                chapter.content = re1
                    .replace_all(&chapter.content, |caps: &Captures| {
                        let path = caps.get(1).unwrap().as_str();
                        process_directive(&map, global_namespace, path, re2.captures_iter(path))
                    })
                    .to_string();
            }
            BookItem::Separator => (),
            BookItem::PartTitle(_) => (),
        });
    }

    serde_json::to_writer(io::stdout(), &book)?;

    Ok(())
}
