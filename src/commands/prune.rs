use inquire::InquireError;

use crate::cli::{GroupUpdateOpts, PruneOpts, RemoveOpts, RenameOpts, RepositoryUpdateOpts, Result, RrhError};
use crate::config::Context;
use crate::db::Database;
use crate::entities::{Group, Repository};

use crate::commands::{group, repository};

use super::{GroupRemoveOpts, RepositoryRemoveOpts};

pub(crate) fn perform_prune(c: &mut Context, opts: PruneOpts) -> Result<bool> {
    let target_groups = find_empty_groups(&c.db);
    let target_repos = find_non_exists_path_repositoreis(&c.db);

    if opts.inquiry {
        match inquire::prompt_confirmation(build_inquiry_message(&target_groups, &target_repos)) {
            Ok(false) => return Ok(false),
            Err(InquireError::IO(e)) => return Err(RrhError::IO(e)),
            Err(e) => return Err(RrhError::Fatal(e.to_string())),
            _ => {},
        };
    }
    match prune_impl(c, target_groups, target_repos) {
        Ok(true) => Ok(!opts.dry_run),
        Ok(false) => Ok(false),
        Err(e) => Err(e),
    }
}

enum RepoOrGroup {
    R(Repository),
    G(Group),
}

pub(crate) fn perform_rename(c: &mut Context, opts: RenameOpts) -> Result<bool> {
    if opts.repository {
        rename_repository(c, &opts.from, &opts.to_name, opts.dry_run)
    } else if opts.group {
        rename_group(c, &opts.from, &opts.to_name, opts.dry_run)
    } else {
        use RepoOrGroup::*;
        match (find_repo_or_group(c, &opts.from), find_repo_or_group(c, &opts.to_name)) {
            (Ok(R(r)), Ok(G(_))) => rename_repository(c, &r.id, &opts.to_name, opts.dry_run),
            (Ok(R(_)), Ok(R(r2))) => Err(RrhError::ToNameExist(r2.id)),
            (Ok(R(r)), Err(RrhError::RepositoryAndGroupNotFound(_))) => rename_repository(c, &r.id, &opts.to_name, opts.dry_run),
            (Ok(R(r)), Err(RrhError::RepositoryAndGroupExists(n))) => Err(RrhError::ToNameExist(r.id)),
            (Ok(G(g)), Ok(R(_))) => rename_group(c, &g.name, &opts.to_name, opts.dry_run),
            (Ok(G(_)), Ok(G(g2))) => Err(RrhError::ToNameExist(g2.name)),
            (Ok(G(g)), Err(RrhError::RepositoryAndGroupNotFound(_))) => rename_group(c, &g.name, &opts.to_name, opts.dry_run),
            (Ok(G(g)), Err(RrhError::RepositoryAndGroupExists(n))) => Err(RrhError::ToNameExist(g.name)),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        }
    }
}

fn rename_repository(c: &mut Context, old_name: &str, new_name: &str, dry_run: bool) -> Result<bool> {
    let r_opts = RepositoryUpdateOpts {
        id: Some(new_name.to_string()),
        path: None,
        description: None,
        groups: vec![],
        new_groups: vec![],
        repository_id: old_name.to_string(),
        dry_run: dry_run,
    };
    repository::perform_update(c, r_opts)
}

fn rename_group(c: &mut Context, old_name: &str, new_name: &str, dry_run: bool) -> Result<bool> {
    let g_opts = GroupUpdateOpts {
        abbrev: None,
        note: None,
        rename_to: Some(new_name.to_string()),
        name: old_name.to_string(),
        dry_run: dry_run,
    };
    group::perform_update(c, g_opts)
}

pub(crate) fn perform_remove(c: &mut Context, opts: RemoveOpts) -> Result<bool> {
    use RepoOrGroup::*;
    let mut errs = vec![];
    for name in opts.targets.clone() {
        let e = match find_repo_or_group(c, &name) {
            Ok(R(_)) => remove_repository(c, name, &opts),
            Ok(G(_)) => remove_group(c, name, &opts),
            Err(e) => Err(e),
        };
        if let Err(e) = e {
            errs.push(e);
        }
    }
    if errs.len() > 0 {
        Err(RrhError::Arrays(errs))
    } else {
        Ok(!opts.dry_run)
    }
}

fn remove_repository(c: &mut Context, name: String, opts: &RemoveOpts) -> Result<bool> {
    let new_opts = RepositoryRemoveOpts {
        ids: vec![name],
        dry_run: opts.dry_run,
    };
    repository::perform_remove(c, new_opts)
}

fn remove_group(c: &mut Context, name: String, opts: &RemoveOpts) -> Result<bool> {
    let new_opts = GroupRemoveOpts {
        force: opts.force,
        args: vec![name],
        dry_run: opts.dry_run,
    };
    group::perform_remove(c, new_opts)
}

fn find_repo_or_group(c: &Context, name: &str) -> Result<RepoOrGroup> {
    match (c.db.find_repository(&name), c.db.find_group(&name)) {
        (Some(r), None) => Ok(RepoOrGroup::R(r)),
        (None, Some(g)) => Ok(RepoOrGroup::G(g)),
        (None, None) => Err(RrhError::RepositoryAndGroupNotFound(name.to_string())),
        (Some(_), Some(_)) => Err(RrhError::RepositoryAndGroupExists(name.to_string())),
    }
}

fn find_non_exists_path_repositoreis(db: &Box<dyn Database>) -> Vec<Repository> {
    db.repositories().unwrap().iter()
        .filter(|r| !r.path.exists())
        .map(|r| r.clone())
        .collect::<Vec<_>>()
}

fn find_empty_groups(db: &Box<dyn Database>) -> Vec<String> {
    db.groups().unwrap().iter()
        .filter(|g| is_empty_group(db, (*g).name.as_str()))
        .map(|g| g.name.clone())
        .collect::<Vec<_>>()
}

fn is_empty_group(db: &Box<dyn Database>, group_name: &str) -> bool {
    db.find_repositories_of(group_name).unwrap().len() == 0
}

fn prune_impl(c: &mut Context, target_groups: Vec<String>, target_repos: Vec<Repository>) -> Result<bool> {
    let mut errs = vec![];
    for group in target_groups {
        if let Err(e) = c.db.delete_group(group) {
            errs.push(e);
        }
    }
    for r in target_repos {
        if let Err(e) = c.db.delete_repository(r.id) {
            errs.push(e);
        }
    }
    if errs.len() > 0 {
        Err(RrhError::Arrays(errs))
    } else {
        Ok(true)
    }
}

fn build_inquiry_message(g: &Vec<String>, r: &Vec<Repository>) -> String {
    use crate::utils::format_humanize;
    format!("found {}, and {}. Do you want to delete them?", 
        format_humanize(g.len(), "empty group", "empty groups"),
        format_humanize(r.len(), "non-exists path repository", "non-exists path repositories"))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::config::Context;
    use crate::entities::Group;

    #[test]
    fn test_prune() {
        let mut c = Context::new_with_path(PathBuf::from("testdata/config.json")).unwrap();
        let opts = crate::cli::PruneOpts { inquiry: false, dry_run: false };
        c.db.register_group(Group::new("pruned_target_group1".to_string())).unwrap();
        c.db.register_group(Group::new("pruned_target_group2".to_string())).unwrap();
        let r = perform_prune(&mut c, opts);

        assert!(r.is_ok());
        assert_eq!(r.unwrap(), true);
        assert_eq!(c.db.groups().unwrap().len(), 1);
    }

    #[test]
    fn test_rename_repo() {
        let mut c = Context::new_with_path(PathBuf::from("testdata/config.json")).unwrap();
        let opts = crate::cli::RenameOpts { repository: false, group: false, from: String::from("helloworld"), to_name: String::from("hw"), dry_run: false };
        let r = perform_rename(&mut c, opts);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), true);
        assert!(c.db.find_repository("hw").is_some());
        assert!(c.db.find_repository("helloworld").is_none());
    }


    #[test]
    fn test_rename_group() {
        let mut c = Context::new_with_path(PathBuf::from("testdata/config.json")).unwrap();
        let opts = crate::cli::RenameOpts { repository: false, group: false, from: String::from("no-group"), to_name: String::from("current"), dry_run: false };
        let r = perform_rename(&mut c, opts);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), true);
        assert!(c.db.find_group("current").is_some());
        assert!(c.db.find_group("no-group").is_none());
    }
}