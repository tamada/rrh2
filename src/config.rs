use chrono::{DateTime, TimeZone};
use chrono_humanize::HumanTime;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::cli::{Result, RrhError};
use crate::db::jsondb::JsonDB;
use crate::db::Database;

pub(crate) struct Context {
    pub(crate) config: Config,
    pub(crate) db: Box<dyn Database>,
}

impl Context {
    pub(crate) fn new() -> Result<Self> {
        Context::new_with_config(Config::new())
    }

    pub(crate) fn new_with_path(path: PathBuf) -> Result<Self> {
        Context::new_with_config(Config::new_with_path(path))
    }

    fn new_with_config(loaded_config: Result<Config>) -> Result<Self> {
        match loaded_config {
            Ok(config) => match load_db(&config) {
                Ok(db) => Ok(Self { config, db }),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }

    pub(crate) fn store(&mut self) -> Result<()> {
        if let Err(e) = store_db(&self.config, &mut self.db) {
            return Err(e);
        }
        store_config(&self.config)
    }
}

fn load_db(config: &Config) -> Result<Box<dyn Database>> {
    match JsonDB::load(config.database_path.clone()) {
        Ok(db) => Ok(Box::new(db)),
        Err(e) => {
            println!("load db error: {:?}", e);
            Err(e)
        }
    }
}

fn store_db(config: &Config, db: &mut Box<dyn Database>) -> Result<()> {
    match File::create(&config.database_path) {
        Ok(file) => db.store(Box::new(file)),
        Err(e) => Err(RrhError::IO(e)),
    }
}

fn store_config(config: &Config) -> Result<()> {
    if let Some(from) = &config.from {
        match File::create(from) {
            Ok(file) => match serde_json::to_writer(file, config) {
                Ok(_) => Ok(()),
                Err(e) => Err(RrhError::Json(e)),
            },
            Err(e) => Err(RrhError::IO(e)),
        }
    } else {
        Err(RrhError::Fatal("config path was not set".into()))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Config {
    pub(crate) home: PathBuf,
    #[serde(rename = "config_path")]
    pub(crate) from: Option<PathBuf>,
    pub(crate) database_path: PathBuf,
    pub(crate) envs: HashMap<String, EnvValue>,
    pub(crate) aliases: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum EnvValue {
    Bool(bool),
    Var(String),
    Value(i32),
}

impl Config {
    pub(crate) fn new() -> Result<Self> {
        dotenv().ok();
        let config_dir = config_dir();
        let config_path = if let Ok(p) = env::var("config_path") {
            PathBuf::from(p)
        } else {
            config_dir.join("config.json")
        };
        load_config(config_path)
    }

    pub(crate) fn new_with_path(config_path: PathBuf) -> Result<Self> {
        dotenv().ok();
        load_config(config_path)
    }

    pub(crate) fn value(&self, key: String) -> Option<EnvValue> {
        if let Some(v) = self.envs.get(&key) {
            Some(v.clone())
        } else {
            match env::var(&key) {
                Ok(v) => Some(EnvValue::Var(v)),
                Err(_) => None,
            }
        }
    }

    pub(crate) fn is_old(&self, time: SystemTime) -> bool {
        let duration = if let Some(EnvValue::Value(t)) =
            self.value("last_access_reload_duration_secs".to_string())
        {
            Duration::from_secs(t as u64)
        } else {
            Duration::from_secs(24 * 60 * 60)
        };
        match time.elapsed() {
            Ok(d) => d > duration,
            Err(_) => true,
        }
    }

    pub(crate) fn to_string(&self, t: SystemTime) -> String {
        match format_time(t, self.value(String::from("last_access_format"))) {
            Some(v) => v,
            None => String::from(""),
        }
    }

    pub fn get_env(&self, key: &str) -> Option<&EnvValue> {
        self.envs.get(key)
    }

    pub fn is_env_value_true(&self, key: &str) -> Option<bool> {
        match self.get_env(key) {
            Some(EnvValue::Bool(b)) => Some(*b),
            Some(EnvValue::Var(s)) => {
                let s = s.to_lowercase();
                Some(s == "true" || s == "yes")
            }
            None => None,
            _ => Some(false),
        }
    }
}

fn system_time_to_datetime(t: SystemTime) -> DateTime<chrono::Local> {
    let (sec, nsec) = match t.duration_since(std::time::UNIX_EPOCH) {
        Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos()),
        Err(e) => { // unlikely but should be handled
            let dur = e.duration();
            let (sec, nsec) = (dur.as_secs() as i64, dur.subsec_nanos());
            if nsec == 0 {
                (-sec, 0)
            } else {
                (-sec - 1, 1_000_000_000 - nsec)
            }
        },
    };
    chrono::Local.timestamp_opt(sec, nsec).unwrap()
}

fn format_time(t: SystemTime, v: Option<EnvValue>) -> Option<String> {
    let dt = system_time_to_datetime(t);
    match v {
        Some(EnvValue::Var(orig)) => {
            let s = orig.to_lowercase();
            if s == "humanize" || s == "relative"  {
                Some(format!("{}", HumanTime::from(dt)))
            } else if s.starts_with("strftime(") && s.ends_with(")") {
                println!("strftime({})", &orig[9..orig.len() - 1]);
                Some(format!("{}", dt.format(&orig[9..orig.len() - 1])))
            } else if s == "iso" || s == "iso8601" {
                Some(format!("{}", dt.format("%+")))
            } else if s == "rfc2822" {
                Some(dt.to_rfc2822())
            } else if s == "rfc3339" {
                Some(dt.to_rfc3339())
            } else {
                Some(dt.to_string())
            }
        },
        None | Some(_) => {
            Some(format!("{}", HumanTime::from(dt)))
        }
    }
}

fn load_config(config_path: PathBuf) -> Result<Config> {
    match load_config_impl(config_path.clone()) {
        Ok(mut c) => {
            c.from = Some(config_path);
            Ok(c)
        }
        Err(e) => Err(e),
    }
}

fn load_config_impl(config_path: PathBuf) -> Result<Config> {
    match std::fs::read_to_string(&config_path) {
        Ok(data) => match serde_json::from_str(&data) {
            Ok(c) => Ok(c),
            Err(e) => Err(RrhError::Json(e)),
        },
        Err(e) => Err(RrhError::IO(e)),
    }
}

fn config_dir() -> PathBuf {
    if let Ok(p) = env::var("config_dir") {
        PathBuf::from(p)
    } else if let Some(p) = dirs::config_dir() {
        p.join("rrh2")
    } else {
        PathBuf::from(".")
    }
}

#[cfg(test)]
mod tests {
    use chrono::Local;

    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_load_config() {
        match Config::new_with_path("testdata/config.json".into()) {
            Ok(c) => {
                assert_eq!(c.database_path, PathBuf::from("testdata/database.json"));
                assert_eq!(c.from, Some(PathBuf::from("testdata/config.json")));
                assert_eq!(c.envs.len(), 8);
                assert_eq!(c.aliases.len(), 2);
            }
            Err(e) => panic!("Error: {:?}", e),
        }
    }

    #[test]
    fn test_systemtime_to_string() {
        assert_eq!(Some(String::from("now")), format_time(SystemTime::now(), None));
        assert_eq!(Some(String::from("a day ago")), format_time(SystemTime::now() - Duration::from_secs(60*60*24), None));
        assert_eq!(Some(String::from("3 days ago")), format_time(SystemTime::now() - Duration::from_secs(3*60*60*24), None));

        let dt = Local.with_ymd_and_hms(2024, 6, 10, 14, 34, 21).unwrap();
        let st = SystemTime::from(dt);
        // https://dtantsur.github.io/rust-openstack/chrono/format/strftime/index.html
        assert_eq!(Some(String::from("2024-06-10 14:34:21")), format_time(st, Some(EnvValue::Var(String::from("strftime(%Y-%m-%d %H:%M:%S)")))));
    }
}