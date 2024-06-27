use std::collections::HashMap;
use std::path::PathBuf;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::cli::{Result, RrhError};
use crate::db::{Database, RefDB};
use crate::entities::{Group, Relation, Repository, RepositoryWithGroups};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct JsonDB {
    last_modified: chrono::DateTime<chrono::Utc>,
    repositories: Vec<Repository>,
    groups: Vec<Group>,
    relations: Vec<Relation>,
}

impl JsonDB {
    pub fn load(path: PathBuf) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(data) => JsonDB::from_str(&data),
            Err(e) => Err(RrhError::IO(e)),
        }
    }

    fn from_str(data: &str) -> Result<Self> {
        match serde_json::from_str(data) {
            Ok(db) => Ok(db),
            Err(e) => Err(RrhError::Json(e)),
        }
    }

    pub fn to_json(&mut self) -> Result<String> {
        self.last_modified = chrono::Utc::now();
        match serde_json::to_string(self) {
            Ok(data) => Ok(data),
            Err(e) => Err(RrhError::Json(e)),
        }
    }
}

impl RefDB for JsonDB {
    fn find_repository(&self, id: &str) -> Option<Repository> {
        self.repositories.iter().find(|r| r.id == id).cloned()
    }

    fn find_repository_with_groups(&self, id: &str) -> Option<RepositoryWithGroups> {
        if let Some(repo) = self.find_repository(id) {
            if let Ok(groups) = self.find_groups_of(id) {
                return Some(RepositoryWithGroups { repo, groups })
            }
        }
        None
    }

    fn find_group(&self, name: &str) -> Option<Group> {
        self.groups.iter().find(|g| g.name == name).cloned()
    }

    fn find_groups_of(&self, id: &str) -> Result<Vec<Group>> {
        let group_names = self
            .relations
            .iter()
            .filter(|r| r.id == id)
            .map(|r| r.group.clone())
            .collect::<Vec<_>>();
        let mut groups = Vec::new();
        for name in group_names {
            if let Some(g) = self.find_group(&name) {
                groups.push(g);
            }
        }
        Ok(groups)
    }

    fn find_repositories_of(&self, group_name: &str) -> Result<Vec<Repository>> {
        let repo_ids = self
            .relations
            .iter()
            .filter(|r| r.group == group_name)
            .map(|r| r.id.clone())
            .collect::<Vec<_>>();
        let mut repositories = Vec::new();
        for id in repo_ids {
            if let Some(r) = self.find_repository(&id) {
                repositories.push(r);
            }
        }
        Ok(repositories)
    }

    fn has_relation(&self, repo_id: &str, group_name: &str) -> bool {
        self.relations
            .iter()
            .any(|r| r.id == repo_id && r.group == group_name)
    }

    fn find_relation(&self, repo_id: &str, group_name: &str) -> Option<Relation> {
        self.relations
            .iter()
            .find(|r| r.id == repo_id && r.group == group_name)
            .cloned()
    }

    fn find_relation_with_group(&self, group_name: &str) -> Vec<Relation> {
        self.relations.iter()
            .filter(|r| r.group == group_name)
            .map(|r| r.clone())
            .collect::<Vec<Relation>>()
    }

    fn find_relation_with_repository(&self, repo_id: &str) -> Vec<Relation> {
        self.relations.iter()
            .filter(|r| r.id == repo_id)
            .map(|r| r.clone())
            .collect::<Vec<Relation>>()
    }

    fn groups(&self) -> Result<Vec<Group>> {
        Ok(self.groups.clone())
    }

    fn group_repositories(&self) -> Result<HashMap<String, Vec<Repository>>> {
        let mut result = HashMap::<String, Vec<Repository>>::new();
        let mut errs = Vec::<RrhError>::new();
        for group in self.groups.iter() {
            match self.find_repositories_of(&group.name) {
                Ok(r) => _ = result.insert(group.name.clone(), r),
                Err(e) => _ = errs.push(e),
            };
        }
        if errs.len() > 0 {
            Err(RrhError::Arrays(errs))
        } else {
            Ok(result)
        }
    }

    fn repositories(&self) -> Result<Vec<Repository>> {
        Ok(self.repositories.clone())
    }
}

impl Database for JsonDB {
    fn register(&mut self, r: Repository, group_names: Vec<String>) -> Result<()> {
        if let Some(_) = self.find_repository(&r.id) {
            return Err(RrhError::RepositoryExists(r.id.clone()));
        }
        for name in group_names.clone() {
            if !self.find_group(&name).is_none() {
                self.register_group(Group::new(name.clone()))?;
            }
        }
        let mut errs = Vec::new();
        for name in group_names {
            if let Err(e) = self.relate(r.id.clone(), name.clone()) {
                errs.push(e);
            }
        }
        self.repositories.push(r);
        if errs.len() > 0 {
            Err(RrhError::Arrays(errs))
        } else {
            Ok(())
        }
    }

    fn register_group(&mut self, g: Group) -> Result<()> {
        if let Some(_) = self.find_group(&g.name) {
            return Err(RrhError::GroupExists(g.name.clone()));
        }
        self.groups.push(g);
        Ok(())
    }

    fn update_group(&mut self, name: String, group: Group) -> Result<()> {
        let old_name = name.clone();
        let new_name = group.name.clone();
        let r = self.groups
            .iter_mut()
            .find(|g| g.name == name)
            .map(|g| *g = group);

        if let Some(_) = r {
            update_relations_all_for_group(self, &old_name, &new_name)
        } else {
            Err(RrhError::GroupNotFound(name))
        }
    }

    fn update_repository(&mut self, id: String, r: Repository) -> Result<()> {
        let new_name = r.id.clone();
        let old_name = id.clone();
        let r = self.repositories
            .iter_mut()
            .find(|repo| repo.id == id)
            .map(|repo| *repo = r);
        if r == None {
            Err(RrhError::RepositoryNotFound(id))
        } else {
            update_relations_all_for_repository(self, &old_name, &new_name)
        }
    }

    fn relate(&mut self, id: String, group_name: String) -> Result<Relation> {
        match self.find_relation(&id, &group_name) {
            Some(relation) => Ok(relation),
            None => {
                let relation = Relation::new(id.clone(), group_name.clone());
                self.relations.push(relation.clone());
                Ok(relation)
            }
        }
    }

    fn delete_relation(&mut self, id: String, group_name: String) -> Result<()> {
        let idx = self
            .relations
            .iter()
            .position(|r| r.id == id && r.group == group_name);
        match idx {
            Some(i) => {
                self.relations.remove(i);
                Ok(())
            }
            None => Err(RrhError::RelationNotFound(id, group_name)),
        }
    }

    fn delete_repository(&mut self, id: String) -> Result<()> {
        match self.repositories.iter().position(|r| r.id == id) {
            Some(i) => {
                self.repositories.remove(i);
                delete_relation_all_for_repository(self, id.clone())
            }
            None => Err(RrhError::RepositoryNotFound(id)),
        }
    }

    fn delete_group(&mut self, group_name: String) -> Result<()> {
        let idx = self.groups.iter().position(|g| g.name == group_name);
        match idx {
            Some(i) => {
                self.groups.remove(i);
                delete_relation_all_for_group(self, group_name.clone())
            }
            None => Err(RrhError::GroupNotFound(group_name)),
        }
    }

    fn store(&mut self, mut out: Box<dyn std::io::Write>) -> Result<()> {
        match self.to_json() {
            Ok(data) => match out.write(data.as_bytes()) {
                Ok(_) => Ok(()),
                Err(e) => Err(RrhError::IO(e)),
            },
            Err(e) => Err(e),
        }
    }
}

fn update_relations_all_for_repository(db: &mut JsonDB, old_name: &str, new_name: &str) -> Result<()> {
    db.relations.iter_mut()
        .filter(|r| r.group == old_name)
        .for_each(|r| r.group = new_name.to_string());
    Ok(())
}

fn update_relations_all_for_group(db: &mut JsonDB, old_name: &str, new_name: &str) -> Result<()> {
    db.relations.iter_mut()
        .filter(|r| r.group == old_name)
        .for_each(|r| r.group = new_name.to_string());
    Ok(())
}

fn delete_relation_all_for_repository(db: &mut JsonDB, id: String) -> Result<()> {
    let mut indexes = relation_indexes(db, |i, r| {
        if r.id == id {
            Some(i)
        } else {
            None
        }
    });
    indexes.reverse();
    for i in indexes {
        db.relations.remove(i);
    }
    Ok(())
}

fn delete_relation_all_for_group(db: &mut JsonDB, group_name: String) -> Result<()> {
    let mut indexes = relation_indexes(db, |i, r| {
        if r.group == group_name {
            Some(i)
        } else {
            None
        }
    });
    indexes.reverse();
    for i in indexes {
        db.relations.remove(i);
    }
    Ok(())
}

fn relation_indexes<F>(db: &JsonDB, f: F) -> Vec<usize> 
        where F: Fn(usize, &Relation) -> Option<usize> {
    db.relations.iter().enumerate()
            .filter_map(|(i, r)| f(i, r))
            .collect::<Vec<usize>>()
}

pub(crate) fn find_orphan_repositories(db: &JsonDB) -> Vec<Repository> {
    let result = db.relations.iter()
        .map(|r| r.id.clone())
        .dedup()
        .collect::<Vec<String>>();
    db.repositories.iter()
        .filter(|r| !result.contains(&(*r).id))
        .map(|r| r.clone())
        .collect::<Vec<Repository>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        match JsonDB::load(PathBuf::from("testdata/database.json")) {
            Ok(db) => {
                assert_eq!(2, db.repositories.len());
                assert_eq!(1, db.groups.len());
                assert_eq!(2, db.relations.len());

                assert_eq!("fibonacci", db.repositories[0].id);
                assert_eq!(
                    "testdata/fibonacci",
                    db.repositories[0].path.to_str().unwrap()
                );

                assert_eq!("helloworld", db.repositories[1].id);
                assert_eq!(
                    "testdata/helloworld",
                    db.repositories[1].path.to_str().unwrap()
                );
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                assert!(false);
            }
        };
    }

    #[test]
    fn test_store() {
        let mut db = JsonDB::load(PathBuf::from("testdata/database.json")).unwrap();
        match db.to_json() {
            Ok(data) => {
                let db2 = JsonDB::from_str(&data).unwrap();
                assert_eq!(db, db2);
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                assert!(false);
            }
        }
    }
}
