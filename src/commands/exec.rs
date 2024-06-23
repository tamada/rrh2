use git_url_parse::GitUrl;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

use crate::cli::{ExecOpts, OpenOpts, OpenTarget, Result, RrhError};
use crate::config::Context;
use crate::entities::Repository;

pub fn perform_exec(context: &Context, c: ExecOpts) -> Result<bool> {
    if c.arguments.len() == 0 {
        return Err(RrhError::Arguments(String::from(
            "(exec) no commands are given",
        )));
    }
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

// ================ functions for open command ================

pub fn perform_open(context: &Context, c: OpenOpts) -> Result<bool> {
    let target = c.target;
    match find_open_targets(context, c.args.clone()) {
        Ok(repos) => {
            if repos.len() == 0 {
                Err(RrhError::Arguments(String::from(
                    "(open) any repositories and groups are not found",
                )))
            } else {
                perform_open_impl(target, repos)
            }
        }
        Err(e) => Err(e),
    }
}

fn find_open_targets(context: &Context, repos_or_groups: Vec<String>) -> Result<Vec<Repository>> {
    let mut result = vec![];
    let mut errs = vec![];
    for group in repos_or_groups.clone() {
        if let Some(g) = context.db.find_group(&group) {
            match context.db.find_repositories_of(&g.name) {
                Ok(rs) => result.extend(rs),
                Err(e) => errs.push(e),
            }
        }
    }
    for repo_name in repos_or_groups.clone() {
        if let Some(r) = context.db.find_repository(&repo_name) {
            result.push(r);
        }
    }
    if errs.len() != 0 {
        Err(RrhError::Arrays(errs))
    } else {
        Ok(result)
    }
}

fn perform_open_impl(target: OpenTarget, repos: Vec<Repository>) -> Result<bool> {
    let mut errs = vec![];
    for repo in repos {
        if let Err(e) = open_repository(&target, &repo) {
            errs.push(e);
        }
    }
    if errs.len() == 0 {
        Ok(false)
    } else {
        Err(RrhError::Arrays(errs))
    }
}

fn open_path(path: PathBuf) -> Result<bool> {
    match open::that(path) {
        Ok(_) => Ok(false),
        Err(e) => Err(RrhError::IO(e)),
    }
}

fn convert_url_to_project_url<F>(url: &str, f: F) -> Result<String>
where
    F: FnOnce(GitUrl) -> Result<String>,
{
    match GitUrl::parse(url.into()) {
        Err(e) => Err(RrhError::Fatal(format!("{:?}", e))),
        Ok(u) => f(u),
    }
}

fn find_remote<'a>(repo: &'a git2::Repository) -> Result<git2::Remote<'a>> {
    repo.find_remote("origin").map_err(|e| RrhError::Git(e))
}

fn open_repository(target: &OpenTarget, repo: &Repository) -> Result<bool> {
    let path = repo.path.canonicalize().map_err(RrhError::IO)?;
    match target {
        OpenTarget::Folder => open_path(path),
        OpenTarget::Webpage => open_webpage(repo, to_project_webpage),
        OpenTarget::Project => open_webpage(repo, to_project_url),
    }
}

fn open_webpage<F>(repo: &Repository, f: F) -> Result<bool>
where
    F: FnOnce(GitUrl) -> Result<String>,
{
    match git2::Repository::open(&repo.path) {
        Err(e) => Err(RrhError::Git(e)),
        Ok(r) => match find_remote(&r) {
            Err(e) => Err(e),
            Ok(r) => open_that(r, f),
        },
    }
}

fn open_that<F>(remote: git2::Remote, f: F) -> Result<bool>
where
    F: FnOnce(GitUrl) -> Result<String>,
{
    if let Some(url) = remote.url() {
        match convert_url_to_project_url(url, f) {
            Ok(url) => open::that(url).map_err(|e| RrhError::IO(e)).map(|_| false),
            Err(e) => Err(e),
        }
    } else {
        Err(RrhError::Git(git2::Error::from_str(
            "remote url is not found",
        )))
    }
}

fn to_project_webpage(gu: GitUrl) -> Result<String> {
    let owner = if let Some(o) = gu.owner {
        o
    } else if let Some(o) = gu.organization {
        o
    } else {
        String::from("")
    };
    Ok(format!("https://{}.github.io/{}", owner, gu.name))
}

fn to_project_url(gu: GitUrl) -> Result<String> {
    let owner = if let Some(o) = gu.owner {
        o
    } else if let Some(o) = gu.organization {
        o
    } else {
        String::from("")
    };
    Ok(format!(
        "https://{}/{}/{}",
        gu.host.unwrap(),
        owner,
        gu.name
    ))
}

// ================ functions for common ================

fn find_target_repositories(
    context: &Context,
    groups: Vec<String>,
    repositories: Vec<String>,
) -> Result<Vec<Repository>> {
    let mut result = vec![];
    let mut errs = vec![];
    for group in groups {
        match context.db.find_repositories_of(&group) {
            Ok(rs) => result.extend(rs),
            Err(e) => errs.push(e),
        }
    }
    for repo_name in repositories {
        match context.db.find_repository(&repo_name) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_git_url() {
        if let Ok(url1) = convert_url_to_project_url("git@github.com/tamada/rrh2", to_project_url) {
            assert_eq!(url1, String::from("https://github.com/tamada/rrh2"));
        }
        if let Ok(url1) =
            convert_url_to_project_url("git@github.com/tamada/rrh2", to_project_webpage)
        {
            assert_eq!(url1, String::from("https://tamada.github.io/rrh2"));
        }
    }
}
