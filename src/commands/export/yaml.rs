use crate::cli::{Result, RrhError};
use crate::commands::export::Exporter;
use crate::commands::export::common::{Status, map_path};
use crate::entities::{Group, Relation, Repository};

pub(super) struct YamlExporter {
    path_mapper: fn(String) -> String,
    status: Status,
}

impl YamlExporter {
    pub(super) fn new(replace_home: bool) -> Self {
        if replace_home {
            YamlExporter { path_mapper: super::home_replacer, status: Status::Before, }
        } else {
            YamlExporter { path_mapper: |s| s, status: Status::Before, }
        }
    }
}

impl Exporter for YamlExporter {
    fn export_header(&mut self, _out: &mut Box<dyn std::io::Write>) -> Result<()> {
        self.status = Status::Header;
        Ok(())
    }

    fn export_repository(&mut self, r: &Repository, out: &mut Box<dyn std::io::Write>) -> Result<()> {
        if self.status == Status::Header {
            let _ = out.write(b"repositories:");
        }
        self.status = Status::Repository;
        let data = r.map(|id,p,d| format!(r##"
  - id: "{}"
    path: "{}"
    description: "{}""##, id, map_path(p, self.path_mapper), d.unwrap_or(String::from(""))));
        match out.write(data.as_bytes()) {
            Err(e) => Err(RrhError::IO(e)),
            Ok(_) => Ok(()), 
        }
    }

    fn export_group(&mut self, g: &Group, out: &mut Box<dyn std::io::Write>) -> Result<()> {
        if self.status == Status::Repository {
            let _ = out.write(b"\ngroups:");
        }
        self.status = Status::Group;
        let data = format!(r##"
  - name: "{}"
    note: "{}"
    abbrev: {}"##, g.name, g.note, g.abbrev.unwrap_or(false));
        match out.write(data.as_bytes()) {
            Err(e) => Err(RrhError::IO(e)),
            Ok(_) => Ok(()),
        }
    }

    fn export_relation(&mut self, r: &Relation, out: &mut Box<dyn std::io::Write>) -> Result<()> {
        if self.status == Status::Group {
            let _ = out.write(b"\nrelations:");
        }
        self.status = Status::Relation;
        let data = format!(r##"
  - id: "{}"
    name: "{}""##, r.id, r.group);
        match out.write(data.as_bytes()) {
            Err(e) => Err(RrhError::IO(e)),
            Ok(_) => Ok(()),
        }
    }

    fn export_footer(&mut self, _out: &mut Box<dyn std::io::Write>) -> Result<()> {
        self.status = Status::Footer;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};
    use crate::commands::export::{Exporter, export};

    #[test]
    fn test_export() {
        let _ = std::fs::create_dir("results");
        let result = {
            let config = crate::config::Context::new_with_path(std::path::PathBuf::from("testdata/config.json")).unwrap();
            let mut exporter = Box::new(super::YamlExporter::new(false)) as Box<dyn Exporter>;
            let mut dest: Box<dyn std::io::Write> = Box::new(File::create("results/export.yaml").unwrap());
            export(&config.db, &mut exporter, &mut dest)
        };

        assert!(result.is_ok());
        let mut dest_file = File::open("results/export.yaml").unwrap();
        let mut contents = String::new();
        dest_file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, r##"repositories:
  - id: "fibonacci"
    path: "testdata/fibonacci"
    description: ""
  - id: "helloworld"
    path: "testdata/helloworld"
    description: ""
groups:
  - name: "no-group"
    note: ""
    abbrev: false
relations:
  - id: "fibonacci"
    name: "no-group"
  - id: "helloworld"
    name: "no-group""##);
        let _ = std::fs::remove_dir_all("results");
    }
}

