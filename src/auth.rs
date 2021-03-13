use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{DirBuilder, File, OpenOptions};
use std::io::{ErrorKind, Read, Result as IoResult, Write};

#[derive(Serialize, Deserialize)]
struct Auth {
    api: Option<APIAuth>,
    login: Option<LoginAuth>,
}

#[derive(Serialize, Deserialize)]
struct APIAuth {
    key: Option<String>,
    secret: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct LoginAuth {
    handle_or_email: Option<String>,
    password: Option<String>,
}

pub fn get_api_key_secret_from_file() -> Result<(String, String), &'static str>
{
    match get_auth_creds_from_file() {
        Ok(Auth { api, .. }) => match api {
            Some(APIAuth {
                key: Some(k),
                secret: Some(s),
            }) => Ok((k, s)),
            _ => Err("missing api credentials in auth.yml"),
        },
        Err(e) => Err(e),
    }
}

pub fn get_login_details_from_file() -> Result<(String, String), &'static str> {
    match get_auth_creds_from_file() {
        Ok(Auth { login, .. }) => match login {
            Some(LoginAuth {
                handle_or_email: Some(h),
                password: Some(p),
            }) => Ok((h, p)),
            _ => Err("missing login credentials in auth.yml"),
        },
        Err(e) => Err(e),
    }
}

fn get_auth_creds_from_file() -> Result<Auth, &'static str> {
    match ProjectDirs::from(crate::NAME_QUL, crate::NAME_ORG, crate::NAME_BIN) {
        Some(proj_dirs) => {
            let f = open_auth_file(proj_dirs, false);
            match f {
                Ok(mut buff) => {
                    let mut s = String::new();
                    let res = buff.read_to_string(&mut s);
                    match res {
                        Err(e) => match e.kind() {
                            ErrorKind::Interrupted => Err(
                                "reading from auth.yml file was interrupted",
                            ),
                            _ => {
                                Err("reading from auth.yml failed unexpectedly")
                            }
                        },
                        Ok(_) => match serde_yaml::from_str::<Auth>(&s) {
                            Ok(a) => Ok(a),
                            Err(_) => Err("failed to parse auth.yml file"),
                        },
                    }
                }
                Err(e) => match e.kind() {
                    ErrorKind::NotFound => {
                        Err("(auth.yml not found) use `caffeine login` to setup
                        API keys and/or login details.")
                    }
                    ErrorKind::PermissionDenied => {
                        Err("could not open auth.yml, permission denied")
                    }
                    _ => Err("could not open auth.yml, unknown reason"),
                },
            }
        }
        None => Err("couldn't find a valid path to store keys"),
    }
}

pub fn set_auth_creds(
    api_key: Option<String>,
    api_secret: Option<String>,
    login_handle: Option<String>,
    login_password: Option<String>,
) -> Result<(), &'static str> {
    match ProjectDirs::from(crate::NAME_QUL, crate::NAME_ORG, crate::NAME_BIN) {
        Some(proj_dirs) => {
            let mut x = Auth {
                api: None,
                login: None,
            };
            if api_key.is_some() || api_secret.is_some() {
                x.api = Some(APIAuth {
                    key: api_key,
                    secret: api_secret,
                });
            }
            if login_handle.is_some() || login_password.is_some() {
                x.login = Some(LoginAuth {
                    handle_or_email: login_handle,
                    password: login_password,
                });
            }
            if let Ok(a) = get_auth_creds_from_file() {
                // If no details provided then use those already stored
                if x.api.is_none() {
                    x.api = a.api;
                }
                if x.login.is_none() {
                    x.login = a.login;
                }
            }
            // If auth file not successfully opened/parsed, then ignore
            // and overwrite all

            // unwrap is probably ok here since serializing errors are very rare
            let s = serde_yaml::to_string(&x).unwrap();

            let f = open_auth_file(proj_dirs, true);
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
                            ErrorKind::Interrupted => {
                                Err("writing to auth.yml file was interrupted")
                            }
                            _ => Err("writing to auth.yml failed unexpectedly"),
                        },
                        Ok(_) => Ok(()),
                    }
                }
                Err(e) => match e.kind() {
                    ErrorKind::NotFound => Err(
                        "could not open auth.yml, parent dir does not exist",
                    ),
                    ErrorKind::PermissionDenied => {
                        Err("could not open auth.yml, permission denied")
                    }
                    _ => Err("could not open auth.yml, unknown reason"),
                },
            }
        }
        None => Err("couldn't find a valid path to store keys"),
    }
}

fn open_auth_file(proj_dirs: ProjectDirs, write: bool) -> IoResult<File> {
    let d = DirBuilder::new()
        .recursive(true)
        .create(proj_dirs.data_dir());
    match d {
        Ok(_) => OpenOptions::new()
            .create(write)
            .write(write)
            .read(true)
            .open(proj_dirs.data_dir().join(crate::AUTH_FILE_NAME)),
        Err(e) => Err(e),
    }
}
