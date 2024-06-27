use std::collections::HashMap;

use tabled::builder::Builder;
use tabled::settings::Style;
use tabled::Table;

use crate::config::{Context, EnvValue};
use crate::{cli::*, utils};
use crate::entities::Group;

pub(crate) fn perform(c: &mut Context, opts: GroupOpts)  -> Result<bool> {
    match opts.subcmd {
        GroupSubCommand::Add(opts) => perform_add(c, &opts),
        GroupSubCommand::List(opts) => perform_list(c, opts),
        GroupSubCommand::Of(opts) => perform_of(c, opts),
        GroupSubCommand::Remove(opts) => perform_remove(c, opts),
        GroupSubCommand::Update(opts) => perform_update(c, opts),
    }
}

fn perform_add(c: &mut Context, opts: &GroupAddOpts) -> Result<bool> {
    let mut errs = vec![];
    for name in opts.names.clone() {
        if let Some(_) = c.db.find_group(&name) {
            errs.push(RrhError::GroupExists(name.clone()));
        }
        let group = Group::new_with(name, opts.note.clone().unwrap_or(String::from("")), Some(opts.abbrev));
        if let Err(e) = c.db.register_group(group) {
            errs.push(e);
        }
    }
    if errs.len() == 0 {
        Ok(!opts.dry_run)
    } else {
        Err(RrhError::Arrays(errs))
    }
}

fn perform_list(c: &Context, opts: GroupListOpts) -> Result<bool> {
    let mut errs = vec![];
    let target = opts.args.clone();
    let groups = if target.len() > 0 {
        let mut groups = vec![];
        for name in target {
            if let Some(g) = c.db.find_group(&name) {
                groups.push(g);
            } else {
                errs.push(RrhError::GroupNotFound(name.clone()));
            }
        }
        groups
    } else {
        match c.db.groups() {
            Ok(r) => r,
            Err(e) => return Err(e),
        }
    };
    let p_opts = &mut opts.p_opts.clone();
    p_opts.update_entries();
    p_opts.update_format(c.config.get_env("print_list_style"));
    print_group(c, groups, p_opts);
    if errs.len() > 0 {
        return Err(RrhError::Arrays(errs));
    } else {
        Ok(false)
    }
}

fn perform_of(c: &Context, opts: GroupOfOpts) -> Result<bool> {
    let mut errs = vec![];
    let mut result = HashMap::<String, Vec<Group>>::new();
    let target = opts.names.clone();
    for name in target {
        match c.db.find_groups_of(&name) {
            Ok(rs) => {
                result.insert(name, rs);
            },
            Err(e) => errs.push(e),
        }
    }
    let p_opts = &mut opts.p_opts.clone();
    p_opts.update_entries();
    p_opts.update_format(c.config.get_env("print_list_style"));
    print_group_of(c, result, p_opts);
    if errs.len() > 0 {
        return Err(RrhError::Arrays(errs));
    } else {
        Ok(false)
    }
}

pub(crate) fn perform_remove(c: &mut Context, opts: GroupRemoveOpts) -> Result<bool> {
    let mut errs = vec![];
    for name in opts.args.clone() {
        if !opts.force {
            let r = c.db.find_relation_with_group(&name);
            if r.len() > 0 {
                errs.push(RrhError::GroupNotEmpty(name.clone()));
                continue;
            }
        }
        if let Err(e) = c.db.delete_group(name.clone()) {
            errs.push(e);
        }
    }
    if errs.len() > 0 {
        Err(RrhError::Arrays(errs))
    } else {
        Ok(!opts.dry_run)
    }
}

pub(crate) fn perform_update(c: &mut Context, opts: GroupUpdateOpts) -> Result<bool> {
    let group = match c.db.find_group(&opts.name) {
        Some(g) => g,
        None => return Err(RrhError::GroupNotFound(opts.name.clone())),
    };
    let new_group = opts.build_new_group(&group);
    if let Err(e) = c.db.update_group(opts.name.clone(), new_group) {
        return Err(e);
    } else {
        Ok(!opts.dry_run)
    }
}

impl GroupUpdateOpts {
    fn build_new_group(&self, group: &Group) -> Group {
        let mut new_group = group.clone();
        if let Some(rename_to) = &self.rename_to {
            new_group.name = rename_to.clone();
        }
        if let Some(note) = &self.note {
            new_group.note = note.clone();
        }
        if let Some(abbrev) = &self.abbrev {
            new_group.abbrev = Some(abbrev.clone());
        }
        new_group
    }
}

fn print_group_of(c: &Context, result: HashMap<String, Vec<Group>>, opts: &GroupPrintingOpts) {
    for (name, groups) in result {
        println!("\"{}\"'s {}:", name, utils::format_humanize(groups.len(), "group", "groups"));
        print_group(c, groups, opts);
    }
}

fn print_group(c: &Context, groups: Vec<Group>, opts: &GroupPrintingOpts) {
    let mut builder = make_table_builder(&opts);
    for group in &groups {
        let mut row = vec![];
        if opts.is_print_target(&GroupEntry::Name) {
            row.push(group.name.clone());
        }
        if opts.is_print_target(&GroupEntry::Note) {
            row.push(group.note.clone());
        }
        if opts.is_print_target(&GroupEntry::Abbrev) {
            row.push(group.abbrev.unwrap_or(false).to_string());
        }
        if opts.is_print_target(&GroupEntry::Count) {
            let count = match c.db.find_repositories_of(&group.name) {
                Ok(rs) => rs.len(),
                Err(_) => 0,
            };
            row.push(count.to_string());
        }
        builder.push_record(row);
    }
    let table = apply_style(builder, &opts.format);
    println!("{}", table.to_string());
}

fn apply_style(builder: Builder, format: &Option<String>) -> Table {
    let mut table = builder.build();
    if let Some(format) = format {
        if format == "psql" {
            table.with(Style::psql());
        } else if format == "ascii" {
            table.with(Style::ascii());
        } else if format == "ascii_rounded" {
            table.with(Style::ascii_rounded());
        } else if format == "empty" {
            table.with(Style::empty());
        } else if format == "blank" {
            table.with(Style::blank());
        } else if format == "markdown" {
            table.with(Style::markdown());
        } else if format == "sharp" {
            table.with(Style::sharp());
        } else if format == "rounded" {
            table.with(Style::rounded());
        } else if format == "modern_rounded" {
            table.with(Style::modern_rounded());
        } else if format == "re_structured_text" {
            table.with(Style::re_structured_text());
        } else if format == "dots" {
            table.with(Style::dots());
        } else if format == "modern" {
            table.with(Style::modern());
        } else if format == "extended" {
            table.with(Style::extended());
        } else if format == "csv" {
            table.with(Style::empty().vertical(','));
        } else {
            table.with(Style::blank());
        }
    } else {
        table.with(Style::blank());
    }
    table
}

fn make_table_builder(opts: &GroupPrintingOpts) -> Builder {
    let mut builder = Builder::new();
    if !opts.no_header {
        let mut header = vec![];
        if opts.is_print_target(&GroupEntry::Name) {
            header.push("Name");
        }
        if opts.is_print_target(&GroupEntry::Note) {
            header.push("Note");
        }
        if opts.is_print_target(&GroupEntry::Abbrev) {
            header.push("Abbrev");
        }
        if opts.is_print_target(&GroupEntry::Count) {
            header.push("Count");
        }
        builder.push_record(header);
    }
    builder
}

impl GroupPrintingOpts {
    fn is_print_target(&self, target: &GroupEntry) -> bool {
        self.entries.contains(&GroupEntry::All) || self.entries.contains(target)
    }

    fn update_entries(&mut self) {
        if self.entries.len() == 0 {
            self.entries = vec![GroupEntry::All];
        }
    }

    fn update_format(&mut self, format: Option<&EnvValue>) {
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


#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_perform_add() {
        let mut context = Context::new_with_path(PathBuf::from("testdata/config.json"))
                .unwrap();
        let opts = GroupAddOpts {
            names: vec![String::from("group1"), String::from("group2")],
            note: Some(String::from("note")),
            abbrev: true,
            dry_run: false,
        };
        let r = perform_add(&mut context, &opts);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), true);
        assert!(context.db.find_group("group1").is_some());
        assert!(context.db.find_group("group2").is_some());

        let r = context.db.groups();
        assert!(r.is_ok());
        assert_eq!(r.unwrap().len(), 3);
    }

    #[test]
    fn test_perform_remove() {
        let mut context = Context::new_with_path(PathBuf::from("testdata/config.json"))
                .unwrap();
        let opts = GroupRemoveOpts {
            force: false,
            args: vec![String::from("no-group")],
            dry_run: false,
        };
        let r = perform_remove(&mut context, opts);
        assert!(r.is_err());
    }

    #[test]
    fn test_perform_remove_force() {
        let mut context = Context::new_with_path(PathBuf::from("testdata/config.json"))
                .unwrap();
        let opts = GroupRemoveOpts {
            force: true,
            args: vec![String::from("no-group")],
            dry_run: false,
        };
        let r = perform_remove(&mut context, opts);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), true);

        let r = context.db.groups();
        assert!(r.is_ok());
        assert_eq!(r.unwrap().len(), 0);
    }

    #[test]
    fn test_perform_update() {
        let mut context = Context::new_with_path(PathBuf::from("testdata/config.json"))
                .unwrap();
        let opts = GroupUpdateOpts {
            rename_to: Some(String::from("current")),
            note: Some(String::from("note")),
            abbrev: Some(false),
            name: String::from("no-group"),
            dry_run: false,
        };
        let r = perform_update(&mut context, opts);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), true);
        assert!(context.db.find_group("current").is_some());

        let r = context.db.groups();
        assert!(r.is_ok());
        assert_eq!(r.unwrap().len(), 1);

        let r = context.db.find_relation_with_group("current");
        assert_eq!(r.len(), 2);
    }
}