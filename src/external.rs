use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::cli::{Result, RrhError};

pub(crate) fn find_and_execute(args: Vec<String>) -> Result<bool> {
    let command = &args[0].clone();
    let args = args[1..].to_vec();
    if let Ok(paths) = env::var("PATH") {
        for path in paths.split(":") {
            if let Some(cmd) = find_command(PathBuf::from(path), command.to_string()) {
                if let Err(e) = execute(cmd, args.clone()) {
                    return Err(e);
                } else {
                    return Ok(false);
                }
            }
        }
    }
    Ok(false)
}

fn execute(cmd: PathBuf, args: Vec<String>) -> Result<()> {
    let result = Command::new(cmd.to_str().unwrap())
        .args(args.clone())
        .output();
    match result {
        Ok(r) => {
            if !r.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&r.stdout));
            }
            if r.status.success() {
                Ok(())
            } else {
                Err(RrhError::ExternalCommand(r.status, format!("{:?} {}", cmd, args.clone().join(" "))))
            }
        },
        Err(e) => Err(RrhError::IO(e)),
    }
}

fn find_command(base: PathBuf, command: String) -> Option<PathBuf> {
    if base.is_dir() {
        let file1 = base.join(format!("rrh2-{}", command));
        if file1.exists() {
            return Some(file1);
        }
        let file2 = base.join(format!("rrh-{}", command));
        if file2.exists() {
            return Some(file2);
        }
    }
    None
}