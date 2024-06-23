use crate::config::Context;
use crate::cli::{Result, RepositorySubCommand, RepositoryOpts};
use crate::commands::{list, add};

use crate::cli::{RepositoryEntry, RepositoryInfoOpts, RepositoryRemoveOpts, RepositoryUpdateOpts};
use crate::entities::{Group, Repository, RepositoryWithGroups};

use super::RrhError;

pub fn perform(c: &mut Context, opts: RepositoryOpts) -> Result<bool> {
    match opts.subcmd {
        RepositorySubCommand::Add(opts) => add::perform_add(c, opts),
        RepositorySubCommand::List(opts) => list::perform_list(c, opts),
        RepositorySubCommand::Info(opts) => perform_info(c, opts),
        RepositorySubCommand::Remove(opts) => perform_remove(c, opts),
        RepositorySubCommand::Update(opts) => perform_update(c, opts),
    }
}

fn perform_info(c: &mut Context, opts: RepositoryInfoOpts) -> Result<bool> {
    let repos = opts.ids.iter()
            .map(|id| c.db.find_repository_with_groups(&id))
            .filter(|r| r.is_some())
            .map(|r| r.unwrap())
            .collect::<Vec<_>>();
    if repos.len() > 0 {
        let mut p_opts = opts.printOpts;
        if p_opts.entries.len() == 0 {
            p_opts.entries = vec![RepositoryEntry::All];
        }
        return list::print_list(repos, &mut c.config, p_opts)
    }
    Ok(false)
}

fn perform_remove(c: &mut Context, opts: RepositoryRemoveOpts) -> Result<bool> {
    if opts.ids.len() > 0 {
        let mut errs = vec![];
        for id in opts.ids {
            if let Err(e) = c.db.delete_repository(id) {
                errs.push(e)
            }
        }
        if errs.len() > 0 {
            return Err(RrhError::Arrays(errs))
        } else {
            Ok(true)
        }
    } else {
        Ok(false)
    }
}

fn perform_update(c: &mut Context, opts: RepositoryUpdateOpts) -> Result<bool> {
    match c.db.find_repository_with_groups(&opts.repository_id) {
        Some(r) => {
            let (new_repo, g) = build_new_repo(c, r.clone(), &opts);
            if opts.renew_groups() {
                if let Err(e) = remove_all_relations(c, &r.groups, &opts.repository_id) {
                    return Err(e)
                }
            }
            repository_update_impl(c, opts.repository_id, new_repo, g)
        },
        None => Err(RrhError::RepositoryNotFound(opts.repository_id.clone()))
    }
}

fn remove_all_relations(c: &mut Context, groups: &Vec<Group>, repo_id: &str) -> Result<()> {
    let mut errs = vec![];
    for g in groups {
        if c.db.has_relation(repo_id, &g.name) {
            if let Err(e) = c.db.delete_relation(repo_id.to_string(), g.name.clone()) {
                errs.push(e);
            }
        }
    }
    if errs.len() > 0 {
        Err(RrhError::Arrays(errs))
    } else {
        Ok(())
    }
}

fn repository_update_impl(c: &mut Context, old_id: String, r: Repository, g: Vec<String>) -> Result<bool> {
    if let Err(e) = c.db.update_repository(old_id, r.clone()) {
        return Err(e)
    }
    let id = r.id.clone();
    let mut errs = vec![];
    for gname in g {
        if let Err(e) = relate_with(c, &gname, &id) {
            errs.push(e);
        }
    }
    if errs.len() > 0 {
        return Err(RrhError::Arrays(errs))
    } else {
        Ok(true)
    }
}

fn relate_with(c: &mut Context, group_name: &str, repo_id: &str) -> Result<()> {
    if !c.db.has_relation(repo_id, group_name) {
        if c.db.find_group(group_name) == None {
            if let Err(e)  = match c.config.is_env_value_true("auto_create_group") {
                Some(true) => c.db.register_group(Group::new(group_name.to_string())),
                _ => Err(RrhError::GroupNotFound(group_name.to_string()))
            } {
                return Err(e)
            }
        }
        if let Err(e) = c.db.relate(repo_id.to_string(), group_name.to_string()) {
            return Err(e)
        }
    }
    Ok(())
}

fn build_new_repo(c: &Context, r: RepositoryWithGroups, opts: &RepositoryUpdateOpts) -> (Repository, Vec<String>) {
    let mut new_repo = opts.build_new_repo(&r.repo);
    let _ = new_repo.last_access(&c.config);
    let new_groups = find_groups(r.groups, opts);
    (new_repo, new_groups)
}

fn find_groups(groups: Vec<Group>, opts: &RepositoryUpdateOpts) -> Vec<String> {
    if opts.renew_groups() {
        opts.new_groups.clone()
    } else {
        let mut result = groups.iter().map(|g| g.name.clone()).collect::<Vec<String>>();
        result.extend(opts.groups.clone());
        result
    }
}

impl RepositoryUpdateOpts {
    fn renew_groups(&self) -> bool {
        self.new_groups.len() > 0
    }

    fn build_new_repo(&self, r: &Repository) -> Repository {
        let mut new_repo = r.clone();
        if let Some(id) = &self.id {
            new_repo.id = id.clone();
        }
        if let Some(p) = &self.path {
            new_repo.path = p.clone();
        }
        if let Some(d) = &self.description {
            new_repo.description = Some(d.clone());
        }
        new_repo
    }
}