use std::path::PathBuf;
use std::process::{Command, ExitStatus};

use crate::cli::{ExecOpts, Result, RrhError};
use crate::config::Context;
use crate::entities::Repository;

pub fn perform(context: &Context, c: ExecOpts) -> Result<bool> {
    match find_target_repositories(
        context,
        c.groups.group_names.clone(),
        c.repositories.repository_ids.clone(),
    ) {
        Ok(repos) => perform_impl(c, repos),
        Err(e) => Err(e),
    }
}

fn perform_impl(c: ExecOpts, repos: Vec<Repository>) -> Result<bool> {
    let command = Cmd::new(c.arguments);
    if repos.len() == 0 {
        execute_command_cwd(&command)
    } else {
        let mut errs = vec![];
        for repo in repos {
            print_header(c.no_header, &repo);
            if let Err(e) = execute_command(&command, repo) {
                errs.push(e);
            }
        }
        if errs.len() == 0 {
            Ok(false)
        } else {
            Err(RrhError::Arrays(errs))
        }
    }
}

fn print_header(no_header_flag: bool, repo: &Repository) {
    if !no_header_flag {
        println!("========== {} ({:?}) ==========", repo.id, repo.path);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Cmd {
    cmd: String,
    args: Vec<String>,
}

impl Cmd {
    fn new(args: Vec<String>) -> Self {
        let cmd = args[0].clone();
        let args = args[1..].to_vec();
        Cmd { cmd, args }
    }

    fn execute(&self, dir: PathBuf) -> Result<i32> {
        match Command::new(self.cmd.clone())
            .args(self.args.clone())
            .current_dir(dir)
            .output()
        {
            Ok(output) => {
                print!("{}", String::from_utf8_lossy(&output.stdout));
                print!("{}", String::from_utf8_lossy(&output.stderr));
                let s = output.status;
                output.status.code().ok_or(build_error(self, s))
            }
            Err(e) => Err(RrhError::IO(e)),
        }
    }
}

fn build_error(c: &Cmd, s: ExitStatus) -> RrhError {
    let mut cmd = vec![c.cmd.clone()];
    cmd.extend(c.args.clone());
    RrhError::ExternalCommand(s, cmd.join(" "))
}

fn execute_command_cwd(cmd: &Cmd) -> Result<bool> {
    match cmd.execute(PathBuf::from(".")) {
        Ok(_) => Ok(false),
        Err(e) => Err(e),
    }
}

fn execute_command(cmd: &Cmd, repo: Repository) -> Result<bool> {
    repo.path
        .canonicalize()
        .map_err(RrhError::IO)
        .and_then(|p| cmd.execute(p).map(|_| false))
}

fn find_target_repositories(
    context: &Context,
    groups: Vec<String>,
    repositories: Vec<String>,
) -> Result<Vec<Repository>> {
    let mut result = vec![];
    let mut errs = vec![];
    for group in groups {
        match context.db.find_repositories_of(group) {
            Ok(rs) => result.extend(rs),
            Err(e) => errs.push(e),
        }
    }
    for repo_name in repositories {
        match context.db.find_repository(repo_name.clone()) {
            Some(r) => result.push(r),
            None => errs.push(RrhError::RepositoryNotFound(repo_name)),
        }
    }
    if errs.len() != 0 {
        Err(RrhError::Arrays(errs))
    } else {
        Ok(result)
    }
}
