use std::collections::HashMap;
use std::io::IsTerminal;
use itertools::Itertools;
use tabled::Table;
use tabled::{builder::Builder, settings::Style};

use crate::cli::{RepositoryEntry, RepositoryListOpts, Result, RrhError};
use crate::config::{self, Config, Context, EnvValue};
use crate::entities::{Repository, RepositoryWithGroups};
use crate::terminal::to_string_in_columns;
use crate::utils::format_humanize;

use super::RepositoryPrintingOpts;

pub(crate) fn perform_list(context: &mut Context, c: RepositoryListOpts) -> Result<bool> {
    let mut errs = Vec::<RrhError>::new();
    let mut result = HashMap::<String, Vec<Repository>>::new();
    if c.groups.len() == 0 {
        match context.db.repositories() {
            Ok(rs) => result = rs,
            Err(e) => _ = errs.push(e),
        }
    } else {
        for group in c.groups {
            match context.db.find_repositories_of(&group) {
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
    print_result(result, context, c.printOpts)
}

pub(crate) fn print_list(repos: Vec<RepositoryWithGroups>, config: &mut Config, p_opts: RepositoryPrintingOpts) -> Result<bool> {
    update_result_style(config, p_opts.format);
    let entries = update_entries(p_opts.entries);
    print_table_repo_group(repos, entries, config, p_opts.no_headers)
}

fn update_entries(entries: Vec<RepositoryEntry>) -> Vec<RepositoryEntry> {
    if entries.contains(&RepositoryEntry::All) {
        return vec![
            RepositoryEntry::Id,
            RepositoryEntry::Groups,
            RepositoryEntry::Path,
            RepositoryEntry::Description,
            RepositoryEntry::LastAccess,
        ];
    } else {
        entries
    }
}

fn update_result_style(config: &mut Config, format: Option<String>) {
    if let Some(f) = format {
        config
            .envs
            .insert(String::from("print_list_style"), EnvValue::Var(f));
    }
}

fn print_result(
    result: HashMap<String, Vec<Repository>>,
    context: &mut config::Context,
    opts: RepositoryPrintingOpts,
) -> Result<bool> {
    let entries = update_entries(opts.entries);
    let config = &mut context.config;
    update_result_style(config, opts.format);
    if entries.len() == 0 && result.len() == 1 {
        print_items_in_columns(
            &RepositoryEntry::Id,
            result.values().next().unwrap().clone(),
            config,
        )
    } else if entries.len() == 1 && result.len() == 1 {
        print_items_in_columns(
            entries.get(0).unwrap(),
            result.values().next().unwrap().clone(),
            config,
        )
    } else {
        for (group_name, repos) in result.iter() {
            let group = match context.db.find_group(&group_name) {
                Some(g) => g,
                None => break,
            };
            if group.is_abbrev() && result.len() > 1 {
                print_abbrev(&entries, repos, group_name, config, opts.no_headers);
            } else {
                print_table(&entries, repos, group_name, &config, opts.no_headers)
            }
        }
        Ok(true)
    }
}

fn is_print_target(entries: &Vec<RepositoryEntry>, e: RepositoryEntry) -> bool {
    entries.iter().any(|x| *x == e)
}

fn build_table_builder(entries: &Vec<RepositoryEntry>, no_header: bool) -> Builder {
    let mut builder = Builder::new();
    if !no_header {
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
        builder.push_record(header);
    }
    builder
}

pub(crate) fn print_table_repo_group(repos: Vec<RepositoryWithGroups>, entries: Vec<RepositoryEntry>, config: &Config, no_header: bool) -> Result<bool> {
    let mut builder = build_table_builder(&entries, no_header);
    repos.iter()
        .map(|r| map_to_vec_repo_group(&entries, r, config))
        .for_each(|v| builder.push_record(v));
    let table = apply_style(builder, config.value(String::from("print_list_style")));
    println!("{}", table.to_string());
    Ok(false)
}

fn print_abbrev(_entries: &Vec<RepositoryEntry>, result: &Vec<Repository>, group_name: &str, config: &config::Config, no_header: bool) {
    let mut builder = Builder::new();
    let record = vec![String::from("Group"), group_name.to_string(), format!("{}", format_humanize(result.len(), "repository", "repositories"))];
    builder.push_record(record);
    let table = apply_style(builder, config.value(String::from("print_list_style")));
    println!("{}", table.to_string());
}

fn print_table(
    entries: &Vec<RepositoryEntry>,
    result: &Vec<Repository>,
    g: &str,
    config: &config::Config,
    no_header: bool,
) {
    let mut builder = build_table_builder(entries, no_header);
    result
        .iter()
        .map(|r| map_to_vec(entries, r, g, config))
        .for_each(|v| builder.push_record(v));
    let table = apply_style(builder, config.value(String::from("print_list_style")));
    println!("{}", table.to_string());
}

fn apply_style(builder: Builder, s: Option<EnvValue>) -> Table {
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
            } else if v == "csv" {
                table.with(Style::empty().vertical(','))
            } else {
                table.with(Style::blank()) // default
            }
        }
        None => table.with(Style::blank()),
    };
    table
}

fn map_to_vec_repo_group(entries: &Vec<RepositoryEntry>, r: &RepositoryWithGroups, c: &config::Config) -> Vec<String> {
    let mut result = vec![];
    if is_print_target(entries, RepositoryEntry::Id) {
        result.push(r.repo.id.clone());
    }
    if is_print_target(entries, RepositoryEntry::Groups) {
        result.push(r.groups.iter().map(|g| g.name.clone()).join(", "));
    }
    if is_print_target(entries, RepositoryEntry::Path) {
        result.push(r.repo.path.to_string_lossy().to_string());
    }
    if is_print_target(entries, RepositoryEntry::Description) {
        result.push(r.repo.description.clone().unwrap_or(String::from("")));
    }
    if is_print_target(entries, RepositoryEntry::LastAccess) {
        result.push(r.repo.last_access_string(c));
    }
    result
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
    if std::io::stdout().is_terminal() {
        println!(
            "{}",
            to_string_in_columns(
                repos
                    .iter()
                    .map(|r| entry.clone().to_string(r, c))
                    .collect()
            )
        );
    } else {
        for r in repos {
            println!("{}", entry.clone().to_string(&r, c));
        }
    }
    Ok(true)
}
