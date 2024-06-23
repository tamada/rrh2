use std::path::PathBuf;

use crate::db::Database;
use crate::entities::Repository;
use crate::config::Context;
use crate::cli::{AddOpts, CloneOpts, Result, RrhError, RepositoryOption};

pub fn perform_add(context: &mut Context, c: AddOpts) -> Result<bool> {
    let mut errs = vec![];
    for path in c.paths.clone() {
        match build_repository_from_path(path, &c.repo) {
            Err(e) => errs.push(e),
            Ok(r) => {
                if let Err(e) = register_repository(&mut context.db, r, c.repo.groups.group_names.clone()) {
                    errs.push(e);
                }
            }
        };
    }
    if errs.is_empty() {
        Ok(true)
    } else {
        Err(RrhError::Arrays(errs))
    }
}

fn register_repository(db: &mut Box<dyn Database>, r: Repository, groups: Vec<String>) -> Result<bool> {
    match db.register(r, groups) {
        Ok(()) => Ok(true),
        Err(e) => Err(e), 
    }
}

fn build_repository_from_path(path: PathBuf, c: &RepositoryOption) -> Result<Repository> {
    if !path.exists() {
        return Err(RrhError::RepositoryPathNotFound(path));
    }
    let path = match path.canonicalize() {
        Ok(p) => p,
        Err(e) => return Err(RrhError::IO(e)),
    };
    let id = match find_id(&path, c.repository_id.clone()) {
        Ok(id) => id,
        Err(e) => return Err(e),
    };
    Ok(Repository::new(id, path, c.description.clone()))
}

fn find_id(path: &PathBuf, repository_id: Option<String>) -> Result<String> {
    if let Some(id) = repository_id {
        Ok(id)
    } else {
        path.file_name()
            .and_then(|f| f.to_str())
            .map(|f| f.to_string())
            .ok_or(RrhError::RepositoryPathNotFound(path.clone()))
    }
}

impl CloneOpts {
    fn repo_path(&self) -> PathBuf {
        let repo_name = self.repo_url.split('/').last().unwrap();
        let repo_path = if repo_name.ends_with(".git") {
            repo_name[..repo_name.len() - 4].to_string()
        } else {
            repo_name.to_string()
        };
        self.dest_dir.join(repo_path)
    }
}

pub fn perform_clone(context: &mut Context, c: CloneOpts) -> Result<bool> {
    let repo = match git2::Repository::clone(&c.repo_url.clone(), c.dest_dir.to_str().unwrap()) {
        Ok(r) => r,
        Err(e) => return Err(RrhError::Git(e))
    };
    let path = repo.path().to_path_buf();
    match build_repository_from_path(path, &c.repo) {
        Err(e) => Err(e),
        Ok(r) => {
            match register_repository(&mut context.db, r, c.repo.groups.group_names.clone()) {
                Err(e) => Err(e),
                Ok(r) => Ok(r),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::GroupSpecifier;

    #[test]
    fn test_clone() {
        let mut context = Context::new_with_path("testdata/config.json".into()).unwrap();
        let cloneOpts = CloneOpts {
            repo_url: "https://github.com/tamada/helloworld".into(),
            dest_dir: PathBuf::from("testdir"),
            repo: RepositoryOption {  repository_id: None, groups: GroupSpecifier{ group_names: vec![] }, description: None },
        };
        let result = perform_clone(&mut context, cloneOpts);
    }
}