use std::path::PathBuf;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::cli::RepositoryEntry;
use crate::config::{self, Config};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Group {
    pub name: String,
    pub note: String,
    pub abbrev: Option<bool>,
}

impl Group {
    pub fn new_with(name: String, note: String, abbrev: Option<bool>) -> Self {
        Self { name, note, abbrev }
    }

    pub fn new(name: String) -> Self {
        Self::new_with(name, "".to_string(), Some(false))
    }

    pub fn is_abbrev(&self) -> bool {
        self.abbrev.unwrap_or(false)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Relation {
    pub id: String,
    pub group: String,
}

impl Relation {
    pub fn new(id: String, group: String) -> Self {
        Self { id, group }
    }
}

impl RepositoryEntry {
    pub fn is_id(&self) -> bool {
        matches!(self, RepositoryEntry::Id) || matches!(self, RepositoryEntry::All)
    }
    pub fn is_path(&self) -> bool {
        matches!(self, RepositoryEntry::Path) || matches!(self, RepositoryEntry::All)
    }
    pub fn is_description(&self) -> bool {
        matches!(self, RepositoryEntry::Description) || matches!(self, RepositoryEntry::All)
    }
    pub fn is_groups(&self) -> bool {
        matches!(self, RepositoryEntry::Groups) || matches!(self, RepositoryEntry::All)
    }
    pub fn is_last_access(&self) -> bool {
        matches!(self, RepositoryEntry::LastAccess) || matches!(self, RepositoryEntry::All)
    }
    pub fn is_all(&self) -> bool {
        matches!(self, RepositoryEntry::All)
    }
    pub fn to_string(self, r: &Repository, config: &config::Config) -> String {
        if self.is_id() {
            r.id.to_string()
        } else if self.is_path() {
            r.path.to_string_lossy().to_string()
        } else if self.is_description() {
            r.description.clone().unwrap_or("".to_string())
        } else if self.is_last_access() {
            r.last_access_string(&config)
        } else {
            "".to_string()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct RepositoryWithGroups {
    pub repo: Repository,
    pub groups: Vec<Group>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Repository {
    pub id: String,
    pub path: PathBuf,
    pub description: Option<String>,
    pub last_access: Option<SystemTime>,
}

impl Repository {
    pub fn new(id: String, path: PathBuf, description: Option<String>) -> Self {
        if let Ok(m) = path.metadata() {
            Self {
                id,
                path,
                description,
                last_access: m.accessed().ok(),
            }
        } else {
            Self {
                id,
                path,
                description,
                last_access: None,
            }
        }
    }

    pub fn last_access_string(&self, config: &config::Config) -> String {
        self.last_access
            .map(|t| config.to_string(t))
            .unwrap_or("".to_string())
    }

    pub fn last_access(&mut self, c: &Config) -> Option<SystemTime> {
        if let Some(t) = self.last_access {
            if c.is_old(t) {
                update_last_access(self)
            }
        } else {
            update_last_access(self)
        }
        self.last_access
    }
}

fn update_last_access(r: &mut Repository) {
    if let Ok(m) = r.path.metadata() {
        r.last_access = m.accessed().ok();
    }
}
