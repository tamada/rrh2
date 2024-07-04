use clap::Parser;
use cli::RrhError::{self, *};

use crate::alias::AliasManager;
use crate::cli::{CliOpts, Result, RrhCommand};
use crate::commands::*;

mod alias;
mod cli;
mod commands;
mod config;
mod db;
mod entities;
mod external;
mod terminal;
mod utils;

fn load_context(opts: &CliOpts) -> Result<config::Context> {
    if let Some(config) = &opts.config_file {
        config::Context::new_with_path(config.clone())
    } else {
        config::Context::new()
    }
}

pub(crate) fn perform(opts: CliOpts) -> Result<()> {
    let mut context = match load_context(&opts) {
        Ok(c) => c,
        Err(e) => return Err(e),
    };
    let store_flag = match opts.command {
        Some(RrhCommand::Add(c)) => perform_add(&mut context, c),
        Some(RrhCommand::Alias(c)) => perform_alias(&mut context, c),
        Some(RrhCommand::Clone(c)) => perform_clone(&mut context, c),
        Some(RrhCommand::Find(c)) => perform_find(&context, c),
        Some(RrhCommand::Exec(c)) => perform_exec(&mut context, c),
        Some(RrhCommand::Export(c)) => perform_export(&mut context, c),
        Some(RrhCommand::Group(c)) => perform_group(&mut context, c),
        Some(RrhCommand::Init(c)) => perform_init(&mut context, c),
        Some(RrhCommand::List(c)) => perform_list(&mut context, c),
        Some(RrhCommand::Open(c)) => perform_open(&mut context, c),
        Some(RrhCommand::Prune(c)) => perform_prune(&mut context, c),
        Some(RrhCommand::Repository(c)) => perform_repository(&mut context, c),
        Some(RrhCommand::Recent(c)) => perform_recent(&context, c),
        Some(RrhCommand::Rename(c)) => perform_rename(&mut context, c),
        Some(RrhCommand::Remove(c)) => perform_remove(&mut context, c),
        None => find_alias_or_external_command(&mut context, opts.args),
    };
    match store_flag {
        Ok(true) => context.store(),
        Ok(false) => Ok(()),
        Err(e) => Err(e),
    }
}

fn find_alias_or_external_command(
    context: &mut config::Context,
    args: Vec<String>,
) -> Result<bool> {
    if let Some(a) = context.config.find(args[0].clone()) {
        if let Err(e) = a.execute(args[1..].to_vec()) {
            return Err(e)
        }
    } else {
        let _ = external::find_and_execute(args);
    }
    Ok(false)
}

fn main() {
    let opts = CliOpts::parse();
    if let Err(e) = perform(opts) {
        print_errors(e);
        std::process::exit(1);
    }
}

fn print_errors(e: RrhError) {
    match e {
        IO(e) => eprintln!("IO error: {}", e),
        Json(e) => eprintln!("JSON error: {}", e),
        Git(e) => eprintln!("Git error: {}", e),
        Arguments(m) => eprintln!("arguments error: {}", m),
        GroupNotFound(name) => eprintln!("{}: group not found", name),
        RepositoryNotFound(name) => eprintln!("{}: repository not found", name),
        RelationNotFound(id, group) => {
            eprintln!("{}: relation not found for group {}", id, group)
        }
        CliOptsInvalid(command, message) => eprintln!("{}: {}", command, message),
        RepositoryExists(name) => eprintln!("{}: repository already exists", name),
        GroupExists(name) => eprintln!("{}: group already exists", name),
        GroupNotEmpty(name) => eprintln!("{}: does not remove group since not empty", name),
        Fatal(message) => eprintln!("internal error: {}", message),
        ExternalCommand(status, command) => eprintln!("{} exit status: {}", command, status),
        Arrays(v) => {
            for item in v {
                print_errors(item)
            }
        }
        RepositoryPathNotFound(path) => eprintln!("{}: repository path not found", path.display()),
        RepositoryAndGroupExists(name) => eprintln!("{}: repository and group both exists", name),
        RepositoryAndGroupNotFound(name) => eprintln!("{}: no repository or group found", name),
        ToNameExist(name) => eprintln!("{}: the to name is occupied", name)
    }
}
