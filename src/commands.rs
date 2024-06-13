use crate::cli::*;
use crate::config::Context;

mod add;
mod alias;
mod list;
mod exec;

pub fn perform_add(context: &mut Context, c: AddOpts) -> Result<bool> {
    add::perform_add(context, c)
}

pub fn perform_alias(context: &mut Context, c: AliasOpts) -> Result<bool> {
    alias::perform(context, c)
}

pub fn perform_clone(context: &mut Context, c: CloneOpts) -> Result<bool> {
    add::perform_clone(context, c)
}

pub fn perform_find(context: &Context, c: FindOpts) -> Result<bool> {
    todo!();
}

pub fn perform_exec(context: &mut Context, c: ExecOpts) -> Result<bool> {
    exec::perform(context, c)
}

pub fn perform_export(context: &mut Context, c: ExportOpts) -> Result<bool> {
    todo!();
}

pub fn perform_group(context: &mut Context, c: GroupOpts) -> Result<bool> {
    todo!();
}

pub fn perform_list(context: &mut Context, c: RepositoryListOpts) -> Result<bool> {
    list::perform_list(context, c)
}

pub fn perform_init(context: &mut Context, c: InitOpts) -> Result<bool> {
    todo!();
}

pub fn perform_open(context: &mut Context, c: OpenOpts) -> Result<bool> {
    todo!();
}

pub fn perform_prune(context: &mut Context) -> Result<bool> {
    todo!();
}

pub fn perform_repository(context: &mut Context, c: RepositoryOpts) -> Result<bool> {
    todo!();
}

pub fn perform_recent(context: &Context, c: RecentOpts) -> Result<bool> {
    todo!();
}
