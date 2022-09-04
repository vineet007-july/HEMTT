use std::io::Read;
use std::path::{Path, PathBuf};

use glob::glob;
use toml::Value::Table;

use crate::HEMTTError;

macro_rules! exec {
    ($c:expr) => {
        if let Err(e) = $c() {
            error!("startup error: {}", e);
        };
    };
}

pub fn startup() {
    exec!(check_git_ignore);
    exec!(deprecated_values);
}

/// Checks for the recommended items in a .gitignore
/// Display a warning if they are not found
fn check_git_ignore() -> Result<(), HEMTTError> {
    if Path::new(".gitignore").exists() {
        let mut data = String::new();
        open_file!(".gitignore")?.read_to_string(&mut data)?;
        let mut ignore = crate::GIT_IGNORE.to_vec();
        for l in data.lines() {
            if let Some(index) = ignore
                .iter()
                .position(|&d| d == l)
                .or_else(|| ignore.iter().position(|&d| d.replace("/*", "/") == l))
            {
                ignore.remove(index);
            }
        }
        for i in ignore {
            warn!(".gitignore is missing recommended value `{}`", i)
        }
    }
    Ok(())
}

fn deprecated_values() -> Result<(), HEMTTError> {
    fn _check(file: PathBuf) -> Result<(), HEMTTError> {
        let items = [
            ("sig_name", "authority"),
            ("signame", "authority"),
            ("keyname", "key_name"),
            ("sigversion", "sig_version"),
            ("headerexts", "header_exts"),
        ];
        let mut data = String::new();
        open_file!(&file)?.read_to_string(&mut data)?;
        for line in data.lines() {
            let value = line.parse::<toml::Value>();
            if let Ok(Table(t)) = value {
                let old = items.iter().find(|x| t.contains_key(x.0));
                if let Some(o) = old {
                    warn!(
                        "deprecated value `{}` in `{}` - use `{}`",
                        o.0,
                        file.display(),
                        o.1
                    )
                }
            }
        }
        Ok(())
    }
    if Path::new("hemtt.toml").exists() {
        _check(PathBuf::from("hemtt.toml"))?;
    } else {
        for entry in glob("./.hemtt/*.toml").expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => _check(path)?,
                Err(e) => error!("{:?}", e),
            }
        }
    }
    Ok(())
}
