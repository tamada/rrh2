use std::collections::HashMap;

use crate::entities::{Group, Relation, Repository, RepositoryWithGroups};
use crate::cli::Result;

pub mod jsondb;

pub trait RefDB {
    /// find a repository by its id
    fn find_repository(&self, id: &str) -> Option<Repository>;
    /// find a repository with its groups by its id
    fn find_repository_with_groups(&self, id: &str) -> Option<RepositoryWithGroups>;
    /// find a group by its name
    fn find_group(&self, name: &str) -> Option<Group>;
    /// find groups related with a repository of given id.
    fn find_groups_of(&self, id: &str) -> Result<Vec<Group>>;
    /// find repositories related with a given group name.
    fn find_repositories_of(&self, group_name: &str) -> Result<Vec<Repository>>;
    /// check if a relation exists
    fn has_relation(&self, repo_id: &str, group_name: &str) -> bool;
    /// find a relation by repository id and group name
    fn find_relation(&self, repo_id: &str, group_name: &str) -> Option<Relation>;
    /// find relations by repository id.
    fn find_relation_with_repository(&self, repo_id: &str) -> Vec<Relation>;
    /// find relations by group name.
    fn find_relation_with_group(&self, group_name: &str) -> Vec<Relation>;
    /// find all groups.
    fn groups(&self) -> Result<Vec<Group>>;
    /// find all repositories. the key of the resultant map is the group name.
    fn group_repositories(&self) -> Result<HashMap<String, Vec<Repository>>>;
    fn repositories(&self) -> Result<Vec<Repository>>;
}

pub trait Database: RefDB {
    fn register(&mut self, r: Repository, group_names: Vec<String>) -> Result<()>;
    fn register_group(&mut self, g: Group) -> Result<()>;
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
