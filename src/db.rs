use std::collections::HashMap;

use crate::entities::{Group, Relation, Repository, RepositoryWithGroups};
use crate::cli::Result;

pub mod jsondb;

pub trait RefDB {
    fn find_repository(&self, id: &str) -> Option<Repository>;
    fn find_repository_with_groups(&self, id: &str) -> Option<RepositoryWithGroups>;
    fn find_group(&self, name: &str) -> Option<Group>;
    fn find_groups_of(&self, id: &str) -> Result<Vec<Group>>;
    fn find_repositories_of(&self, group_name: &str) -> Result<Vec<Repository>>;
    fn has_relation(&self, repo_id: &str, group_name: &str) -> bool;
    fn find_relation(&self, repo_id: &str, group_name: &str) -> Option<Relation>;
    fn groups(&self) -> Result<Vec<Group>>;
    fn repositories(&self) -> Result<HashMap<String, Vec<Repository>>>;
}

pub trait Database: RefDB {
    fn register(&mut self, r: Repository, group_names: Vec<String>) -> Result<()>;
    fn register_group(&mut self, g: Group) -> Result<()>;
    fn prune(&mut self, ) -> Result<()>;
    fn update_group(&mut self, name: String, group: Group) -> Result<()>;
    fn update_repository(&mut self, id: String, r: Repository) -> Result<()>;
    fn relate(&mut self, id: String, group_name: String) -> Result<Relation>;
    fn delete_relation(&mut self, id: String, group_name: String) -> Result<()>;
    fn delete_repository(&mut self, id: String) -> Result<()>;
    fn delete_group(&mut self, group_name: String) -> Result<()>;
    fn store(&mut self, out: Box<dyn std::io::Write>) -> Result<()>;
}

pub trait Exportable {
    fn export(&self, out: Box<dyn std::io::Write>) -> Result<()>;
}
