use std::io::Write;

use rust_embed::{Embed, EmbeddedFile};

use crate::config::Context;
use crate::cli::{InitOpts, Result, ShellName};

#[derive(Embed)]
#[folder = "assets"]
#[prefix = "assets/"]
struct Asset;

pub fn perform(_: &Context, c: InitOpts) -> Result<bool> {
    let shell = shell_to_string(c.shell_name);
    if !c.without_cdrrh {
        print_asset("cdrrh", shell.clone());
        print_asset("_cdrrh", shell.clone());
    }
    if !c.without_rrhfzf {
        print_asset("rrhfzf", shell.clone());
    }
    if !c.without_rrhpeco {
        print_asset("rrhpeco", shell.clone());
    }
    let _ = std::io::stdout().flush();
    Ok(false)
}

fn print_asset(name: &str, shell: String) {
    let asset_path = format!("assets/{}/{}", shell, name);
    if let Some(asset) = Asset::get(&asset_path) {
        print_asset_impl(asset);
    } else {
        eprintln!("Asset not found: {}", asset_path);
    }
}

fn print_asset_impl(asset: EmbeddedFile) {
    let _ = std::io::stdout().write(asset.data.as_ref());
}

fn shell_to_string(shell: ShellName) -> String {
    match shell {
        ShellName::Bash => "bash".to_string(),
        ShellName::Zsh => "zsh".to_string(),
        ShellName::Fish => "fish".to_string(),
        ShellName::Elvish => "elvish".to_string(),
        ShellName::Powershell => "powershell".to_string(),
    }
}