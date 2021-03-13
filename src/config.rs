use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{DirBuilder, File, OpenOptions};
use std::io::{ErrorKind, Read, Result as IoResult, Write};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub default_user: Option<String>,
    pub default_program_type_id: Option<i64>,
}

pub fn get_config() -> Result<Config, &'static str> {
    get_config_from_file()
}

fn get_config_from_file() -> Result<Config, &'static str> {
    match ProjectDirs::from(crate::NAME_QUL, crate::NAME_ORG, crate::NAME_BIN) {
        Some(proj_dirs) => {
            let f = open_config_file(proj_dirs, false);
            match f {
                Ok(mut buff) => {
                    let mut s = String::new();
                    let res = buff.read_to_string(&mut s);
                    match res {
                        Err(e) => match e.kind() {
                            ErrorKind::Interrupted => Err(
                                "reading from config.yml file was interrupted",
                            ),
                            _ => Err(
                                "reading from config.yml failed unexpectedly",
                            ),
                        },
                        Ok(_) => match serde_yaml::from_str::<Config>(&s) {
                            Ok(a) => Ok(a),
                            Err(_) => Err("failed to parse config.yml file"),
                        },
                    }
                }
                Err(e) => match e.kind() {
                    ErrorKind::NotFound => {
                        Err("(config.yml not found) use `caffeine config` to \
                        setup defaults.")
                    }
                    ErrorKind::PermissionDenied => {
                        Err("could not open config.yml, permission denied")
                    }
                    _ => Err("could not open config.yml, unknown reason"),
                },
            }
        }
        None => Err("couldn't find a valid path to place config at"),
    }
}

pub fn set_config(
    default_user: Option<String>,
    default_program_type_id: Option<i64>,
) -> Result<(), &'static str> {
    match ProjectDirs::from(crate::NAME_QUL, crate::NAME_ORG, crate::NAME_BIN) {
        Some(proj_dirs) => {
            let mut x = Config {
                default_user: None,
                default_program_type_id: None,
            };
            if default_user.is_some() || default_program_type_id.is_some() {
                x = Config {
                    default_user,
                    default_program_type_id,
                };
            }
            if let Ok(a) = get_config_from_file() {
                // If no details provided then use those already stored
                if x.default_user.is_none() {
                    x.default_user = a.default_user;
                }
                if x.default_program_type_id.is_none() {
                    x.default_program_type_id = a.default_program_type_id;
                }
            }
            // If config file not successfully opened/parsed, then ignore
            // and overwrite all

            // unwrap is probably ok here since serializing errors are very rare
            let s = serde_yaml::to_string(&x).unwrap();

            let f = open_config_file(proj_dirs, true);
            match f {
                Ok(mut buff) => {
                    // errors for both set_len and write can be handled at the
                    // same time
                    let mut res = buff.set_len(0);
                    if res.is_ok() {
                        res = buff.write_all(s.as_bytes());
                    }
                    match res {
                        Err(e) => match e.kind() {
                            ErrorKind::Interrupted => Err(
                                "writing to config.yml file was interrupted",
                            ),
                            _ => {
                                Err("writing to config.yml failed unexpectedly")
                            }
                        },
                        Ok(_) => Ok(()),
                    }
                }
                Err(e) => match e.kind() {
                    ErrorKind::NotFound => Err(
                        "could not open config.yml, parent dir does not exist",
                    ),
                    ErrorKind::PermissionDenied => {
                        Err("could not open config.yml, permission denied")
                    }
                    _ => Err("could not open config.yml, unknown reason"),
                },
            }
        }
        None => Err("couldn't find a valid path to store keys"),
    }
}

fn open_config_file(proj_dirs: ProjectDirs, write: bool) -> IoResult<File> {
    let d = DirBuilder::new()
        .recursive(true)
        .create(proj_dirs.config_dir());
    match d {
        Ok(_) => OpenOptions::new()
            .create(write)
            .write(write)
            .read(true)
            .open(proj_dirs.config_dir().join(crate::CONF_FILE_NAME)),
        Err(e) => Err(e),
    }
}
