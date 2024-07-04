use crate::commands::export::Exporter;
use crate::cli::Result;
use crate::entities::{Group, Relation, Repository};

use super::common::Status;

pub(super) struct JsonExporter {
    path_mapper: fn(String) -> String,
    status: Status,
}

impl JsonExporter {
    pub(super) fn new(replace_home: bool) -> Self {
        if replace_home {
            JsonExporter { path_mapper: super::home_replacer, status: Status::Before }
        } else {
            JsonExporter { path_mapper: |s| s, status: Status::Before }
        }
    }
}

impl Exporter for JsonExporter {
    fn export_header(&mut self, out: &mut Box<dyn std::io::Write>) -> Result<()> {
        let _ = out.write(b"{");
        self.status = Status::Header;
        Ok(())
    }

    fn export_repository(&mut self, r: &Repository, out: &mut Box<dyn std::io::Write>) -> Result<()> {
        let _ = if self.status == Status::Header {
            out.write(b"\"repositories\":[")
        } else {
            out.write(b",")
        };
        self.status = Status::Repository;
        let desc = r.description.clone()
            .map(|d| format!(",\"description\":\"{}\"", d))
            .unwrap_or(String::from(""));
        let path = r.path.to_str()
                .map(|s| s.to_string())
                .map(self.path_mapper);
        let _ = out.write(format!("{{\"id\":\"{}\",\"path\":\"{}\"{}}}", r.id, path.unwrap_or(String::from("")), desc).as_bytes());
        Ok(())
    }

    fn export_group(&mut self, g: &Group, out: &mut Box<dyn std::io::Write>) -> Result<()> {
        let _ = if self.status == Status::Repository {
            out.write(b"],\"groups\":[")
        } else {
            out.write(b",")
        };
        self.status = Status::Group;
        let _ = out.write(format!(r##"{{"name":"{}","note":"{}","abbrev":{}}}"##, g.name, g.note, g.abbrev.unwrap_or(false)).as_bytes());
        Ok(())
    }

    fn export_relation(&mut self, r: &Relation, out: &mut Box<dyn std::io::Write>) -> Result<()> {
        let _ = if self.status == Status::Group {
            out.write(b"],\"relations\":[")
        } else {
            out.write(b",")
        };
        self.status = Status::Relation;
        let _ = out.write(format!(r##"{{"id":"{}","group":"{}"}}"##, r.id, r.group).as_bytes());
        Ok(())
    }

    fn export_footer(&mut self, out: &mut Box<dyn std::io::Write>) -> Result<()> {
        let _ = out.write(b"]}");
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
            let mut exporter = Box::new(super::JsonExporter::new(false)) as Box<dyn Exporter>;
            let mut dest: Box<dyn std::io::Write> = Box::new(File::create("results/export.json").unwrap());    
            export(&config.db, &mut exporter, &mut dest)
        };

        assert!(result.is_ok());
        let mut dest_file = File::open("results/export.json").unwrap();
        let mut contents = String::new();
        dest_file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, r##"{"repositories":[{"id":"fibonacci","path":"testdata/fibonacci"},{"id":"helloworld","path":"testdata/helloworld"}],"groups":[{"name":"no-group","note":"","abbrev":false}],"relations":[{"id":"fibonacci","group":"no-group"},{"id":"helloworld","group":"no-group"}]}"##);
        let _ = std::fs::remove_dir_all("results");
    }
}
