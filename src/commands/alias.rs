use crate::alias::{Alias, AliasManager};
use crate::cli::{AliasOpts, Result, RrhError};
use crate::config::Context;

pub(crate) enum Mode {
    List,
    Register,
    Upadte,
    Remove,
    Execute,
}

impl AliasOpts {
    pub(crate) fn find_mode(&self) -> Mode {
        if self.update {
            Mode::Upadte
        } else if self.remove {
            Mode::Remove
        } else if self.arguments.len() == 0 {
            Mode::List
        } else {
            Mode::Register
        }
    }

    pub(crate) fn validate(&self, context: &Context) -> Result<Mode> {
        let mut errs = vec![];
        if self.update && self.remove {
            errs.push(RrhError::CliOptsInvalid(
                "alias".into(),
                "Cannot update and remove at the same time".into(),
            ));
        }
        let mode = self.find_mode();
        match &mode {
            Mode::Register => validate_register(self, &context.config, &mut errs),
            Mode::Upadte => validate_update(self, &context.config, &mut errs),
            Mode::Remove => validate_remove(self, &context.config, &mut errs),
            Mode::Execute | Mode::List => {}
        }
        if errs.len() == 0 {
            Ok(mode)
        } else {
            Err(RrhError::Arrays(errs))
        }
    }

    pub(crate) fn build_alias(&self) -> Option<Alias> {
        if self.arguments.len() == 0 {
            None
        } else {
            self.alias
                .clone()
                .map(|name| Alias::new(name, self.arguments.clone()))
        }
    }
}

fn validate_remove(c: &AliasOpts, manager: &impl AliasManager, errs: &mut Vec<RrhError>) {
    validate_alias_name(c.alias.clone(), errs, "alias_remove".into());
    if let Some(alias) = &c.alias {
        if manager.find(alias.clone()).is_none() {
            errs.push(RrhError::CliOptsInvalid(
                "alias_remove".into(),
                format!("{}: alias not found", alias),
            ))
        }
    }
}

fn validate_register(c: &AliasOpts, manager: &impl AliasManager, errs: &mut Vec<RrhError>) {
    validate_alias_name(c.alias.clone(), errs, "alias_register".into());
    if c.arguments.len() == 0 {
        errs.push(RrhError::CliOptsInvalid(
            "alias_register".into(),
            "No commands provided".into(),
        ));
    }
    if let Some(a) = &c.alias {
        if manager.find(a.clone()).is_some() {
            errs.push(RrhError::CliOptsInvalid(
                "alias_register".into(),
                format!("{}: already exist alias", a),
            ))
        }
    }
}

fn validate_alias_name(c: Option<String>, errs: &mut Vec<RrhError>, command: String) {
    if c.is_none() {
        errs.push(RrhError::CliOptsInvalid(
            command.into(),
            "No alias provided".into(),
        ))
    }
}

fn validate_update(c: &AliasOpts, manager: &impl AliasManager, errs: &mut Vec<RrhError>) {
    validate_alias_name(c.alias.clone(), errs, "alias_update".into());
    if let Some(alias_name) = &c.alias {
        if manager.find(alias_name.clone()).is_none() {
            errs.push(RrhError::CliOptsInvalid(
                "alias_update".into(),
                format!("{}: alias not found", alias_name),
            ))
        }
    }
}

fn perform_list(manager: &impl AliasManager) -> Result<bool> {
    for alias in manager.iterator() {
        println!("{} = {}", alias.name, alias.commands.join(" "));
    }
    Ok(false)
}

fn perform_register(manager: &mut impl AliasManager, c: AliasOpts) -> Result<bool> {
    if let Some(alias) = c.build_alias() {
        match manager.register(alias) {
            Ok(_) => Ok(true),
            Err(e) => Err(e),
        }
    } else {
        unreachable!("unreachable since already check in validate_register")
    }
}

fn perform_update(manager: &mut impl AliasManager, c: AliasOpts) -> Result<bool> {
    if let Some(alias) = c.build_alias() {
        match manager.update(alias) {
            Ok(_) => Ok(true),
            Err(e) => Err(e),
        }
    } else {
        unreachable!("unreachable since already check in validate_register")
    }
}

fn perform_remove(manager: &mut impl AliasManager, c: AliasOpts) -> Result<bool> {
    if let Some(alias_name) = c.alias {
        match manager.delete(alias_name) {
            Ok(_) => Ok(true),
            Err(e) => Err(e),
        }
    } else {
        unreachable!("unreachable since already check in validate_remove")
    }
}

pub fn perform(context: &mut Context, c: AliasOpts) -> Result<bool> {
    match c.validate(&context) {
        Err(e) => Err(e),
        Ok(mode) => match mode {
            Mode::List => perform_list(&context.config),
            Mode::Register => perform_register(&mut context.config, c),
            Mode::Upadte => perform_update(&mut context.config, c),
            Mode::Remove => perform_remove(&mut context.config, c),
            Mode::Execute => unreachable!("Mode::Execute is never reach here!"),
        },
    }
}
