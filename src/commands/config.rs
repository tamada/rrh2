use crate::config::{Config, Context, EnvValue};
use crate::cli::{ConfigOpts, Result, RrhError};

pub(super) fn perform_config(context: &mut Context, c: ConfigOpts) -> Result<bool> {
    let dry_run_flag = c.dry_run;
    let r = match c.mode() {
        Mode::List => list_config(&mut context.config, c),
        Mode::Set => set_config(&mut context.config, c),
        Mode::Unset => unset_config(&mut context.config, c),
    };
    match r {
        Ok(_) => Ok(!dry_run_flag),
        Err(e) => Err(e),
    }
}

fn print_config_item(key: &str, value: &EnvValue) {
    match value {
        EnvValue::Bool(s) => println!("{} = {}", key, s),
        EnvValue::Var(s) => println!("{} = \"{}\"", key, s),
        EnvValue::Value(v) => println!("{} = {}", key, v),
    }
}

fn list_config(config: &mut Config, opts: ConfigOpts) -> Result<()> {
    if let Some(key) = opts.name {
        if let Some(v) = config.value(&key) {
            print_config_item(&key, &v);
        } else {
            return Err(RrhError::ConfigNotFound(key));
        }
    } else {
        for (k, v) in config.envs.iter() {
            print_config_item(k, v);
        }
    }
    Ok(())
}

fn set_config(config: &mut Config, c: ConfigOpts) -> Result<()> {
    if let (Some(key), Some(value)) = (c.name, c.value) {
        match config.value(&key) {
            Some(EnvValue::Value(_)) => put_int_value(config, key, value),
            Some(EnvValue::Var(_)) => put_string_value(config, key, value),
            Some(EnvValue::Bool(_))  => put_bool_value(config, key, value),
            None => put_guessed_value(config, key, value),
        }
    } else {
        unreachable!("name and value are required");
    }
}

fn put_guessed_value(config: &mut Config, key: String, value: String) -> Result<()> {
    let v = value.to_lowercase();
    if let Ok(v) = value.parse::<i32>() {
        config.envs.insert(key, EnvValue::Value(v));
    } else if v == "true" || v == "yes" {
        let _ = put_bool_value(config, key, value);
    } else {
        let _ = put_string_value(config, key, value);
    }
    Ok(())
}

fn put_bool_value(config: &mut Config, key: String, value: String) -> Result<()> {
    let v = value.to_lowercase();
    if v == "true" || v == "yes" {
        config.envs.insert(key, EnvValue::Bool(true));
    } else if v == "false" || v == "no" {
        config.envs.insert(key, EnvValue::Bool(false));
    } else {
        config.envs.insert(key, EnvValue::Var(value));
    }
    Ok(())
}

fn put_string_value(config: &mut Config, key: String, value: String) -> Result<()> {
    config.envs.insert(key, EnvValue::Var(value));
    Ok(())
}

fn put_int_value(config: &mut Config, key: String, value: String) -> Result<()> {
    if let Ok(v) = value.parse::<i32>() {
        config.envs.insert(key, EnvValue::Value(v));
        Ok(())
    } else {
        config.envs.insert(key, EnvValue::Var(value));
        Ok(())
    }
}

fn unset_config(config: &mut Config, c: ConfigOpts) -> Result<()> {
    if let Some(name) = c.name {
        config.envs.remove(&name);
        Ok(())
    } else {
        unreachable!("name is required");
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Mode {
    List,
    Set,
    Unset,
}

impl ConfigOpts {
    fn mode(&self) -> Mode {
        match (&self.name, &self.value) {
            (Some(_), Some(_)) => Mode::Set,
            (Some(_), None) => {
                if self.remove {
                    Mode::Unset
                } else {
                    Mode::List
                }
            },
            (None, _) => Mode::List,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode() {
        let opts = ConfigOpts { name: None, value: None, remove: false, dry_run: false };
        assert_eq!(opts.mode(), Mode::List);

        let opts = ConfigOpts { name: Some(String::from("name")), value: None, remove: false, dry_run: false };
        assert_eq!(opts.mode(), Mode::List);

        let opts = ConfigOpts { name: Some(String::from("name")), value: None, remove: true, dry_run: false };
        assert_eq!(opts.mode(), Mode::Unset);

        let opts = ConfigOpts { name: Some(String::from("name")), value: Some(String::from("value")), remove: false, dry_run: false };
        assert_eq!(opts.mode(), Mode::Set);
    }
}