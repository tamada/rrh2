use std::collections::HashMap;
use tabled::{builder::Builder, settings::Style};

use crate::cli::{RepositoryEntry, RepositoryListOpts, Result, RrhError};
use crate::config::{self, Context, EnvValue};
use crate::entities::Repository;
use crate::terminal::to_string_in_columns;

pub(crate) fn perform_list(context: &mut Context, c: RepositoryListOpts) -> Result<bool> {
    let mut errs = Vec::<RrhError>::new();
    let mut result = HashMap::<String, Vec<Repository>>::new();
    if c.groups.len() == 0 {
        match context.db.repositories() {
            Ok(rs) => result = rs,
            Err(e) => _ = errs.push(e),
        }
    } else {
        result = HashMap::<String, Vec<Repository>>::new();
        for group in c.groups {
            match context.db.find_repositories_of(group.clone()) {
                Ok(rs) => {
                    result.insert(group, rs);
                }
                Err(e) => errs.push(e),
            }
        }
    }
    if errs.len() != 0 {
        return Err(RrhError::Arrays(errs));
    }
    print_result(c.entries, result, &context.config)
}

fn print_result(
    entries: Vec<RepositoryEntry>,
    result: HashMap<String, Vec<Repository>>,
    config: &config::Config,
) -> Result<bool> {
    if entries.len() == 0 && result.len() == 1 {
        print_items_in_columns(&RepositoryEntry::Id, 
            result.values().next().unwrap().clone(), config)
    } else if entries.len() == 1 && result.len() == 1 {
        print_items_in_columns(
            entries.get(0).unwrap(),
            result.values().next().unwrap().clone(),
            config,
        )
    } else {
        for (group, repos) in result.iter() {
            print_table(&entries, repos, group, &config);
        }
        Ok(true)
    }
}

fn is_print_target(entries: &Vec<RepositoryEntry>, e: RepositoryEntry) -> bool {
    entries.iter().any(|x| *x == e || x.is_all())
}

fn print_table(
    entries: &Vec<RepositoryEntry>,
    result: &Vec<Repository>,
    g: &str,
    config: &config::Config,
) {
    let mut header = vec![];
    if is_print_target(&entries, RepositoryEntry::Id) {
        header.push(String::from("ID"));
    }
    if is_print_target(&entries, RepositoryEntry::Groups) {
        header.push(String::from("Groups"));
    }
    if is_print_target(&entries, RepositoryEntry::Path) {
        header.push(String::from("Path"));
    }
    if is_print_target(&entries, RepositoryEntry::Description) {
        header.push(String::from("Description"))
    }
    if is_print_target(&entries, RepositoryEntry::LastAccess) {
        header.push(String::from("Last Access"));
    }
    let mut builder = Builder::new();
    builder.push_record(header);
    result
        .iter()
        .map(|r| map_to_vec(entries, r, g, config))
        .for_each(|v| builder.push_record(v));
    let string = parse_style(builder, config.value(String::from("print_list_style")));
    println!("{}", string);
}

fn parse_style(builder: Builder, s: Option<EnvValue>) -> String {
    let mut table = builder.build();
    match s {
        Some(EnvValue::Bool(_)) | Some(EnvValue::Value(_)) => table.with(Style::blank()),
        Some(EnvValue::Var(v)) => {
            let v = v.to_lowercase();
            if v == "psql" {
                table.with(Style::psql())
            } else if v == "ascii" {
                table.with(Style::ascii())
            } else if v == "ascii_rounded" {
                table.with(Style::ascii_rounded())
            } else if v == "empty" {
                table.with(Style::empty())
            } else if v == "blank" {
                table.with(Style::blank())
            } else if v == "markdown" {
                table.with(Style::markdown())
            } else if v == "sharp" {
                table.with(Style::sharp())
            } else if v == "rounded" {
                table.with(Style::rounded())
            } else if v == "modern_rounded" {
                table.with(Style::modern_rounded())
            } else if v == "re_structured_text" {
                table.with(Style::re_structured_text())
            } else if v == "dots" {
                table.with(Style::dots())
            } else if v == "modern" {
                table.with(Style::modern())
            } else if v == "extended" {
                table.with(Style::extended())
            } else {
                table.with(Style::blank()) // default
            }
        }
        None => table.with(Style::blank()),
    };
    table.to_string()
}

fn map_to_vec(
    entries: &Vec<RepositoryEntry>,
    r: &Repository,
    g: &str,
    c: &config::Config,
) -> Vec<String> {
    let mut result = vec![];
    if is_print_target(entries, RepositoryEntry::Id) {
        result.push(r.id.clone());
    }
    if is_print_target(entries, RepositoryEntry::Groups) {
        result.push(g.to_string());
    }
    if is_print_target(entries, RepositoryEntry::Path) {
        result.push(r.path.to_string_lossy().to_string());
    }
    if is_print_target(entries, RepositoryEntry::Description) {
        result.push(r.description.clone().unwrap_or(String::from("")));
    }
    if is_print_target(entries, RepositoryEntry::LastAccess) {
        result.push(r.last_access_string(c));
    }
    result
}

fn print_items_in_columns(
    entry: &RepositoryEntry,
    repos: Vec<Repository>,
    c: &config::Config,
) -> Result<bool> {
    println!(
        "{}",
        to_string_in_columns(
            repos
                .iter()
                .map(|r| entry.clone().to_string(r, c))
                .collect()
        )
    );
    Ok(true)
}
