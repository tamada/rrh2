use clap::Parser;
use std::iter::Iterator;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::cli::{CliOpts, Result, RrhError};

pub(crate) trait AliasManager {
    fn iterator(&self) -> impl Iterator<Item=Alias>;
    fn find(&self, alias_name: String) -> Option<Alias>;
    fn register(&mut self, alias: Alias) -> Result<()>;
    fn update(&mut self, alias: Alias) -> Result<()>;
    fn delete(&mut self, alias_name: String) -> Result<()>;
}

impl AliasManager for Config {
    fn find(&self, alias_name: String) -> Option<Alias> {
        self.aliases.find(alias_name)
    }

    fn register(&mut self, alias: Alias) -> Result<()> {
        self.aliases.register(alias)
    }

    fn update(&mut self, alias: Alias) -> Result<()> {
        self.aliases.update(alias)
    }

    fn delete(&mut self, alias_name: String) -> Result<()> {
        self.aliases.delete(alias_name)
    }

    fn iterator(&self) -> impl Iterator<Item=Alias> {
        self.aliases.iterator()
    }
}

impl AliasManager for HashMap<String, Vec<String>> {
    fn iterator(&self) -> impl Iterator<Item=Alias> {
        let mut v = self.iter()
            .map(|(name, commands)| Alias::new(name.clone(), commands.clone()))
            .collect::<Vec<Alias>>();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v.into_iter()
    }

    fn find(&self, alias_name: String) -> Option<Alias> {
        match self.get(&alias_name) {
            Some(commands) => Some(Alias::new(alias_name, commands.clone())),
            None => None,
        }
    }
    
    fn register(&mut self, alias: Alias) -> Result<()> {
        self.insert(alias.name.clone(), alias.commands.clone());
        Ok(())
    }
    
    fn update(&mut self, alias: Alias) -> Result<()> {
        if !self.contains_key(&alias.name) {
            Err(RrhError::CliOptsInvalid(
                "alias_update".into(),
                format!("{}: alias not found", alias.name),
            ))
        } else {
            self.register(alias.clone())
        }
    }
    
    fn delete(&mut self, alias_name: String) -> Result<()> {
        self.remove_entry(&alias_name);
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Alias {
    pub(crate) name: String,
    pub(crate) commands: Vec<String>,
}

impl Alias {
    pub(crate) fn new(name: String, commands: Vec<String>) -> Self {
        Self { name, commands }
    }

    pub(crate) fn execute(&self, args: Vec<String>) -> Result<()> {
        let mut new_args = vec![String::from("rrh2")];
        new_args.extend(self.commands.clone());
        new_args.extend(args.clone());
        crate::perform(CliOpts::parse_from(new_args))
    }
}

