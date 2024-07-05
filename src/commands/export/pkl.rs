use crate::commands::export::Exporter;
use crate::cli::Result;
use crate::entities::{Group, Relation, Repository};
use crate::commands::export::common::{Status, map_path};

pub(super) struct PklExporter {
    replace_home: bool,
    status: Status,
    path_mapper: fn(String) -> String,
}

impl PklExporter {
    pub(super) fn new(replace_home: bool) -> Self {
        if replace_home {
            PklExporter { replace_home: replace_home, path_mapper: home_replacer, status: Status::Before }
        } else {
            PklExporter { replace_home: replace_home, path_mapper: |s| s, status: Status::Before }
        }
    }
}

fn home_replacer(s: String) -> String {
    let home = std::env::var("HOME").unwrap();
    s.replace(&home, "\\(home)").to_string()
}

impl Exporter for PklExporter {
    fn export_header(&mut self, out: &mut Box<dyn std::io::Write>) -> Result<()> {
        if self.replace_home {
            let _ = out.write(b"user_home = read(\"env:HOME\")\n\n");
        }
        write_definitions(out);
        self.status = Status::Header;
        Ok(())
    }

    fn export_repository(&mut self, r: &Repository, out: &mut Box<dyn std::io::Write>) -> Result<()> {
        if self.status == Status::Header {
            let _ = out.write(b"repositories {");
        }
        self.status = Status::Repository;
        let _ = out.write(r.map(|id,p,d| format!(r##"
  new Repository {{
    id = "{}"
    path = "{}"
    description = "{}"
  }}"##, id, map_path(p, self.path_mapper), d.unwrap_or(String::from("")))).as_bytes());
        Ok(())
    }

    fn export_group(&mut self, g: &Group, out: &mut Box<dyn std::io::Write>) -> Result<()> {
        if self.status == Status::Repository {
            let _ = out.write(b"\n}\ngroups {");
        }
        self.status = Status::Group;
        let _ = out.write(format!(r##"
  new Group {{
    name = "{}"
    note = "{}"
    abbrev = {}
  }}"##, g.name, g.note, g.abbrev.unwrap_or(false)).as_bytes());
        Ok(())
    }

    fn export_relation(&mut self, r: &Relation, out: &mut Box<dyn std::io::Write>) -> Result<()> {
        if self.status == Status::Group {
            let _ = out.write(b"\n}\nrelations {");
        }
        self.status = Status::Relation;
        let _ = out.write(format!(r##"
  new Relation {{
    id = "{}"
    group = "{}"
  }}"##, r.id, r.group).as_bytes());
        Ok(())
    }

    fn export_footer(&mut self, out: &mut Box<dyn std::io::Write>) -> Result<()> {
        let _ = out.write(b"\n}");
        self.status = Status::Footer;
        Ok(())
    }
}

fn write_definitions(out: &mut Box<dyn std::io::Write>) {
    let _ = out.write(r##"class Repository {
    id: String
    path: String
    description: String
}
class Group {
    name: String
    note: String
    abbrev: Boolean
}
class Relation {
    id: String
    group: String
}
"##.as_bytes());
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
            let mut exporter = Box::new(super::PklExporter::new(false)) as Box<dyn Exporter>;
            let mut dest: Box<dyn std::io::Write> = Box::new(File::create("results/export.pkl").unwrap());    
            export(&config.db, &mut exporter, &mut dest)
        };

        assert!(result.is_ok());
        let mut dest_file = File::open("results/export.pkl").unwrap();
        let mut contents = String::new();
        dest_file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, r##"class Repository {
    id: String
    path: String
    description: String
}
class Group {
    name: String
    note: String
    abbrev: Boolean
}
class Relation {
    id: String
    group: String
}
repositories {
  new Repository {
    id = "fibonacci"
    path = "testdata/fibonacci"
    description = ""
  }
  new Repository {
    id = "helloworld"
    path = "testdata/helloworld"
    description = ""
  }
}
groups {
  new Group {
    name = "no-group"
    note = ""
    abbrev = false
  }
}
relations {
  new Relation {
    id = "fibonacci"
    group = "no-group"
  }
  new Relation {
    id = "helloworld"
    group = "no-group"
  }
}"##);
    }
}