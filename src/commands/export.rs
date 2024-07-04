use std::path::Path;

use crate::cli::{ExportOpts, Result, RrhError};
use crate::config::Context;
use crate::db::Database;
use crate::entities::{Group, Relation, Repository};

mod common;
mod json;
mod pkl;
mod yaml;

enum Format {
    Json,
    Yaml,
    Pkl,
}

trait Exporter {
    fn export_header(&mut self, out: &mut Box<dyn std::io::Write>) -> Result<()>;
    fn export_repository(&mut self, r: &Repository, out: &mut Box<dyn std::io::Write>) -> Result<()>;
    fn export_group(&mut self, g: &Group, out: &mut Box<dyn std::io::Write>) -> Result<()>;
    fn export_relation(&mut self, r: &Relation, out: &mut Box<dyn std::io::Write>) -> Result<()>;
    fn export_footer(&mut self, out: &mut Box<dyn std::io::Write>) -> Result<()>;
}

pub(crate) fn perform_export(c: &Context, opts: ExportOpts) -> Result<bool> {
    if let Err(e) = opts.validate() {
        return Err(e);
    };

    let mut dest = opts.open_dest();
    match build_exporter(opts.format(), opts.no_replace_home) {
        Ok(mut exporter) => {
            match export(&c.db, &mut exporter, &mut dest) {
                Ok(_) => Ok(true),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

fn export(db: &Box<dyn Database>, exporter: &mut Box<dyn Exporter>, 
            out: &mut Box<dyn std::io::Write>) -> Result<()> {
    exporter.export_header(out)?;
    for r in db.repositories().unwrap() {
        exporter.export_repository(&r, out)?;
    }
    for g in db.groups().unwrap() {
        exporter.export_group(&g, out)?;
    }
    for r in db.relations().unwrap() {
        exporter.export_relation(&r, out)?;
    }
    exporter.export_footer(out)
}

fn build_exporter(format: Option<Format>, no_replace_home: bool) -> Result<Box<dyn Exporter>> {
    match format {
        Some(Format::Json) => 
            Ok(Box::new(json::JsonExporter::new(!no_replace_home)) as Box<dyn Exporter>),
        Some(Format::Yaml) =>
            Ok(Box::new(yaml::YamlExporter::new(!no_replace_home)) as Box<dyn Exporter>),
        Some(Format::Pkl) =>
            Ok(Box::new(pkl::PklExporter::new(!no_replace_home)) as Box<dyn Exporter>),
        None => 
            Err(RrhError::Arguments(String::from("unknown export format"))),
    }
}

pub(crate) fn home_replacer(s: String) -> String {
    let home = std::env::var("HOME").unwrap();
    s.replace(&home, "${HOME}").to_string()
}

impl ExportOpts {
    fn validate(&self) -> Result<()> {
        let mut errs = vec![];
        if let Err(e) = Format::validate(&self.format) {
            errs.push(e);
        }
        if let Err(e) = validate_dest(&self.dest, self.overwrite) {
            errs.push(e);
        }

        if errs.len() > 0 {
            Err(RrhError::Arrays(errs))
        } else {
            Ok(())
        }
    }

    fn format(&self) -> Option<Format> {
        match self.format {
            Some(ref f) => Format::parse(f),
            None => Format::from_filename(&self.dest),
        }
    }

    fn open_dest(&self) -> Box<dyn std::io::Write> {
        if let Some(dest) = &self.dest {
            if dest == "-" {
                return Box::new(std::io::stdout()) as Box<dyn std::io::Write>;
            } else {
                Box::new(std::fs::File::create(dest).unwrap()) as Box<dyn std::io::Write>
            }
        } else {
            Box::new(std::io::stdout()) as Box<dyn std::io::Write>
        }
    }
}

impl Format {
    fn from_filename(file_name: &Option<String>) -> Option<Self> {
        if let Some(d) = file_name {
            let d = d.to_lowercase();
            if d.ends_with(".json") {
                Some(Format::Json)
            } else if d.ends_with(".yaml") || d.ends_with(".yml") {
                Some(Format::Yaml)
            } else if d.ends_with(".pkl") {
                Some(Format::Pkl)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn parse(f: &str) -> Option<Self> {
        let fl = f.to_lowercase();
        if fl == "json" {
            Some(Format::Json)
        } else if fl == "yaml" {
            Some(Format::Yaml)
        } else if fl == "pkl"{
            Some(Format::Pkl)
        } else {
            None
        }
    }

    fn validate(f: &Option<String>) -> Result<()> {
        if let Some(f) = f {
            let fl = f.to_lowercase();
            if fl != "json" && fl != "yaml" && fl != "pkl" {
                return Err(RrhError::Arguments(format!("{}: unknown export format", f)));
            }
        }
        Ok(())
    }
}


fn validate_dest(dest: &Option<String>, overwrite: bool) -> Result<()> {
    if let Some(dest) = dest {
        if Path::new(dest).exists() && !overwrite {
            return Err(RrhError::Arguments(format!("{}: file exists", dest)));
        }
    } 
    Ok(())
}
