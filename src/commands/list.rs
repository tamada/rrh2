use std::collections::HashMap;
use std::io::IsTerminal;
use itertools::Itertools;
use tabled::Table;
use tabled::{builder::Builder, settings::Style};

use crate::cli::{RepositoryEntry, RepositoryListOpts, Result, RrhError, RepositoryPrintingOpts};
use crate::config::{self, Config, Context, EnvValue};
use crate::entities::{Repository, RepositoryWithGroups};
use crate::terminal::to_string_in_columns;
use crate::utils::format_humanize;

use super::{FindOpts, RecentOpts};

pub(crate) fn perform_recent(context: &Context, mut c: RecentOpts) -> Result<bool> {
    let result = context.db.repositories().unwrap().iter()
        .sorted_by(|&a, &b| a.last_access.cmp(&b.last_access))
        .take(c.number.unwrap_or(5))
        .map(|r| build_repo_with_group(r, context))
        .collect::<Vec<_>>();
    let p_opts = &mut c.p_opts;
    p_opts.update_entries();
    p_opts.update_format(context.config.value("print_list_style"));
    print_table_repo_group(result, &p_opts, &context.config)
}

pub(crate) fn perform_find(context: &Context, mut c: FindOpts) -> Result<bool> {
    let r = match find_impl(context, &c) {
        Ok(repos) => repos,
        Err(err) => return Err(err),
    };
    let p_opts = &mut c.p_opts;
    p_opts.update_entries();
    p_opts.update_format(context.config.value("print_list_style"));
    print_table_repo_group(r, &p_opts, &context.config)
}

pub(crate) fn find_impl(context: &Context, c: &FindOpts) -> Result<Vec<RepositoryWithGroups>> {
    let result = match context.db.repositories() {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let result = result.iter()
        .filter(|r| is_target_repository(*r, &c))
        .map(|r| build_repo_with_group(r, context))
        .collect::<Vec<_>>();
    Ok(result)
}

fn is_target_repository(r: &Repository, c: &FindOpts) -> bool {
    let mut iter = c.keywords.iter();
    let f = |w| r.id.contains(w)
                || r.path.to_string_lossy().contains(w)
                || r.description.as_ref().unwrap_or(&String::from("")).contains(w);
    if c.and {
        iter.all(f)
    } else {
        iter.any(f)
    }
}

fn build_repo_with_group(r: &Repository, context: &Context) -> RepositoryWithGroups {
    let groups = context.db.find_relation_with_repository(&r.id).iter()
        .map(|r| context.db.find_group(&r.group).unwrap())
        .collect::<Vec<_>>();
    RepositoryWithGroups {
        repo: r.clone(),
        groups,
    }
}

pub(crate) fn perform_list(context: &Context, mut c: RepositoryListOpts) -> Result<bool> {
    let mut errs = Vec::<RrhError>::new();
    let mut result = HashMap::<String, Vec<Repository>>::new();
    if c.groups.len() == 0 {
        match context.db.group_repositories() {
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
    let p_opts = &mut c.p_opts;
    p_opts.update_entries();
    p_opts.update_format(context.config.value("print_list_style"));
    print_result(result, p_opts, context)
}

pub(crate) fn print_list(repos: Vec<RepositoryWithGroups>, config: &mut Config, p_opts: &mut RepositoryPrintingOpts) -> Result<bool> {
    p_opts.update_format(config.value("print_list_style"));
    print_table_repo_group(repos, p_opts, config)
}

fn print_result(
    result: HashMap<String, Vec<Repository>>,
    opts: &RepositoryPrintingOpts,
    context: &config::Context,
) -> Result<bool> {
    if opts.entries.len() == 1 && result.len() == 1 {
        print_items_in_columns(
            opts.entries.get(0).unwrap(),
            result.values().next().unwrap().clone(),
            &context.config,
        )
    } else {
        for (group_name, repos) in result.iter() {
            let group = match context.db.find_group(&group_name) {
                Some(g) => g,
                None => break,
            };
            if group.is_abbrev() && result.len() > 1 {
                print_abbrev(repos, opts, group_name, &context.config)
            } else {
                print_table(repos, opts, group_name, &context.config)
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

pub(crate) fn print_table_repo_group(repos: Vec<RepositoryWithGroups>, opts: &RepositoryPrintingOpts, config: &Config) -> Result<bool> {
    let mut builder = build_table_builder(&opts.entries, opts.no_headers);
    repos.iter()
        .map(|r| map_to_vec_repo_group(&opts.entries, r, config))
        .for_each(|v| builder.push_record(v));
    let table = apply_style(builder, &opts.format);
    println!("{}", table.to_string());
    Ok(false)
}

fn print_abbrev(result: &Vec<Repository>, opts: &RepositoryPrintingOpts, group_name: &str, _config: &config::Config) {
    let mut builder = Builder::new();
    let record = vec![String::from("Group"), group_name.to_string(), format!("{}", format_humanize(result.len(), "repository", "repositories"))];
    builder.push_record(record);
    let table = apply_style(builder, &opts.format);
    println!("{}", table.to_string());
}

fn print_table(
    result: &Vec<Repository>,
    opts: &RepositoryPrintingOpts,
    g: &str,
    config: &config::Config
) {
    let mut builder = build_table_builder(&opts.entries, opts.no_headers);
    result
        .iter()
        .map(|r| map_to_vec(&opts.entries, r, g, config))
        .for_each(|v| builder.push_record(v));
    let table = apply_style(builder, &opts.format);
    println!("{}", table.to_string());
}

fn apply_style(builder: Builder, s: &Option<String>) -> Table {
    let mut table = builder.build();
    match s {
        Some(v) => {
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

impl RepositoryPrintingOpts {
    fn update_entries(&mut self) {
        if self.entries.contains(&RepositoryEntry::All) {
            self.entries = vec![
                RepositoryEntry::Id,
                RepositoryEntry::Groups,
                RepositoryEntry::Path,
                RepositoryEntry::Description,
                RepositoryEntry::LastAccess,
            ]
        } else if self.entries.len() == 0 {
            self.entries = vec![RepositoryEntry::Id]
        }
    }

    fn update_format(&mut self, format: Option<EnvValue>) {
        let availables = vec![
                "psql", "ascii", "ascii_rounded", "empty", "blank", "markdown", "sharp", "rounded", 
                "modern_rounded", "re_structured_text", "dots", "modern", "extended", "csv",
        ].iter().map(|s| s.to_string()).collect::<Vec<String>>();
        self.format = if let Some(f) = &self.format {
            let f = f.to_lowercase();
            if availables.contains(&f) {
                Some(f)
            } else {
                Some(String::from("blank"))
            }
        } else {
            match format {
                Some(EnvValue::Var(f)) => {
                    let f = f.to_lowercase();
                    if availables.contains(&f) {
                        Some(f)
                    } else {
                        Some(String::from("blank"))
                    }
                },
                _ => Some(String::from("blank")),
            }
        };
    }
}
