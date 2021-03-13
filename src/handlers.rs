#![allow(clippy::many_single_char_names)]

use clap::{value_t, values_t, ArgMatches};
use codeforces_api::requests::*;
use codeforces_api::responses::CFResult;
use codeforces_api::Error as ApiError;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::path::Path;

use crate::{auth, config, submit};

fn get_from_api<T: CFAPIRequestable>(args: &ArgMatches, x: &T) -> CFResult {
    let (k, s) = get_api_key_secret(args);
    match x.get(&k, &s) {
        Ok(s) => s,
        Err(e) => exit_with_error(e),
    }
}

fn get_from_api_raw<T: CFAPIRequestable>(args: &ArgMatches, x: &T) -> String {
    let (k, s) = get_api_key_secret(args);
    match x.get_raw(&k, &s) {
        Ok(s) => s,
        Err(e) => exit_with_error(e),
    }
}

pub fn get_optional_arg_of_type<T: std::str::FromStr>(
    args: &ArgMatches,
    name: &str,
) -> Option<T> {
    match args.is_present(name) {
        true => {
            Some(value_t!(args.value_of(name), T).unwrap_or_else(|e| e.exit()))
        }
        false => None,
    }
}

pub fn get_optional_args_of_type<T: std::str::FromStr>(
    args: &ArgMatches,
    name: &str,
) -> Option<Vec<T>> {
    match args.is_present(name) {
        true => Some(
            values_t!(args.values_of(name), T).unwrap_or_else(|e| e.exit()),
        ),
        false => None,
    }
}

pub fn exit_with_usage(matches: &ArgMatches) -> ! {
    eprintln!("{}", matches.usage());
    std::process::exit(1);
}

pub fn exit_with_error<E: std::fmt::Display>(e: E) -> ! {
    eprintln!("Error: {}", e);
    std::process::exit(2);
}

pub fn get_api_key_secret(args: &ArgMatches) -> (String, String) {
    let key = get_optional_arg_of_type(args, "key");
    let sec = get_optional_arg_of_type(args, "secret");
    match (key, sec) {
        (Some(k), Some(s)) => (k, s),
        _ => {
            let res = auth::get_api_key_secret_from_file();
            match res {
                Ok(ks) => ks,
                Err(e) => exit_with_error(e),
            }
        }
    }
}

fn get_login_details(args: &ArgMatches) -> (String, String) {
    let handle = get_optional_arg_of_type(args, "handle");
    let password = get_optional_arg_of_type(args, "password");
    match (handle, password) {
        (Some(h), Some(p)) => (h, p),
        _ => {
            let res = auth::get_login_details_from_file();
            match res {
                Ok(hp) => hp,
                Err(e) => exit_with_error(e),
            }
        }
    }
}

pub fn login_command(args: &ArgMatches) {
    println!("{}", crate::AUTH_HELP_MSG);
    let key = get_optional_arg_of_type(args, "key");
    let sec = get_optional_arg_of_type(args, "secret");
    let handle = get_optional_arg_of_type(args, "handle");
    let password = get_optional_arg_of_type(args, "password");

    if (key.is_some() && sec.is_some())
        || (handle.is_some() && password.is_some())
    {
        match auth::set_auth_creds(key, sec, handle, password) {
            Ok(()) => {
                println!("successfully set credentials");
            }
            Err(e) => exit_with_error(e),
        }
    } else {
        let mut k = String::new();
        let mut s = String::new();
        let mut h = String::new();
        let mut p = String::new();

        print!("API key (leave blank to ignore): ");
        stdout().flush().expect("unable to flush stdout?");
        let res = stdin().read_line(&mut k);
        if let Err(e) = res {
            exit_with_error(e);
        }
        k = k.trim().to_string();
        // If key input was empty, then don't ask for secret
        if !k.is_empty() {
            print!("API secret (required): ");
            stdout().flush().expect("unable to flush stdout?");
            let res = stdin().read_line(&mut s);
            s = s.trim().to_string();
            if let Err(e) = res {
                exit_with_error(e);
            }
            if s.is_empty() {
                exit_with_error(
                    "API secret field is required (leave `API key`\
                    field blank to ignore)"
                        .to_string(),
                );
            }
        }
        let k = if !k.is_empty() { Some(k) } else { None };
        let s = if !s.is_empty() { Some(s) } else { None };

        print!("handle or email (leave blank to ignore): ");
        stdout().flush().expect("unable to flush stdout?");
        let res = stdin().read_line(&mut h);
        if let Err(e) = res {
            exit_with_error(e);
        }
        h = h.trim().to_string();
        // If username input was empty, then don't ask for password
        if !h.is_empty() {
            print!("password (required): ");
            stdout().flush().expect("unable to flush stdout?");
            let res = stdin().read_line(&mut p);
            if let Err(e) = res {
                exit_with_error(e);
            }
            p = p.trim().to_string();

            if p.is_empty() {
                exit_with_error(
                    "password field is required (leave `handle or \
                    email` field blank to ignore)"
                        .to_string(),
                );
            }
        }
        let h = if !h.is_empty() { Some(h) } else { None };
        let p = if !p.is_empty() { Some(p) } else { None };

        println!("saving keys to auth.yml");
        match auth::set_auth_creds(k, s, h, p) {
            Ok(()) => {
                println!("successfully set credentials");
            }
            Err(e) => exit_with_error(e),
        }
    }
}

pub fn config_command(args: &ArgMatches) {
    let d_u = get_optional_arg_of_type(args, "defaultuser");
    let d_pti = get_optional_arg_of_type(args, "defaultprogramtypeid");

    if d_u.is_some() || d_pti.is_some() {
        match config::set_config(d_u, d_pti) {
            Ok(()) => {
                println!("successfully saved settings");
            }
            Err(e) => exit_with_error(e),
        }
    } else {
        let mut u = String::new();
        let mut p = String::new();

        print!("default user handle (leave blank to ignore): ");
        stdout().flush().expect("unable to flush stdout?");
        let res = stdin().read_line(&mut u);
        if let Err(e) = res {
            exit_with_error(e);
        }
        u = u.trim().to_string();

        println!("default program type id (leave blank to ignore): ");
        print!("ID LANGUAGE\n{}\n: ", crate::PROGRAM_TYPE_ID_HELP);
        stdout().flush().expect("unable to flush stdout?");
        let res = stdin().read_line(&mut p);
        if let Err(e) = res {
            exit_with_error(e);
        }
        p = p.trim().to_string();

        let u = if !u.is_empty() { Some(u) } else { None };
        let p: Option<i64> = if !p.is_empty() {
            match p.parse() {
                Ok(n) => Some(n),
                Err(_) => exit_with_error("id must be an integer, aborting"),
            }
        } else {
            None
        };

        println!("saving settings to config.yml");
        match config::set_config(u, p) {
            Ok(()) => {
                println!("successfully saved settings");
            }
            Err(e) => exit_with_error(e),
        }
    }
}

pub fn blogentry_command(matches: &ArgMatches) {
    match matches.subcommand() {
        ("comments", Some(args)) => {
            let i = get_optional_arg_of_type(args, "BLOGENTRYID").unwrap();
            let x = CFBlogEntryCommand::Comments { blog_entry_id: i };
            if args.is_present("raw") {
                println!("{}", get_from_api_raw(args, &x));
            } else {
                println!("{}", get_from_api(args, &x))
            }
        }
        ("view", Some(args)) => {
            let i = get_optional_arg_of_type(args, "BLOGENTRYID").unwrap();
            let x = CFBlogEntryCommand::View { blog_entry_id: i };
            if args.is_present("raw") {
                println!("{}", get_from_api_raw(args, &x));
            } else {
                println!("{}", get_from_api(args, &x))
            }
        }
        _ => {
            exit_with_usage(matches);
        }
    }
}

pub fn contest_command(matches: &ArgMatches) {
    match matches.subcommand() {
        ("hacks", Some(args)) => {
            let i = get_optional_arg_of_type(args, "CONTESTID").unwrap();
            let x = CFContestCommand::Hacks { contest_id: i };
            if args.is_present("raw") {
                println!("{}", get_from_api_raw(args, &x));
            } else {
                println!("{}", get_from_api(args, &x))
            }
        }
        ("list", Some(args)) => {
            let b = args.is_present("gym");
            let x = CFContestCommand::List { gym: Some(b) };
            if args.is_present("raw") {
                println!("{}", get_from_api_raw(args, &x));
            } else {
                println!("{}", get_from_api(args, &x))
            }
        }
        ("ratingchanges", Some(args)) => {
            let i = get_optional_arg_of_type(args, "CONTESTID").unwrap();
            let x = CFContestCommand::RatingChanges { contest_id: i };
            if args.is_present("raw") {
                println!("{}", get_from_api_raw(args, &x));
            } else {
                println!("{}", get_from_api(args, &x))
            }
        }
        ("standings", Some(args)) => {
            let i = get_optional_arg_of_type(args, "CONTESTID").unwrap();
            let f = get_optional_arg_of_type(args, "from");
            let n = get_optional_arg_of_type(args, "count");
            let h = get_optional_args_of_type(args, "handles");
            let r = get_optional_arg_of_type(args, "room");
            let s = args.is_present("showunofficial");
            let t = args.is_present("dontfetchtestcases");
            let x = CFContestCommand::Standings {
                contest_id: i,
                from: f,
                count: n,
                handles: h,
                room: r,
                show_unofficial: Some(s),
            };
            if args.is_present("raw") {
                println!("{}", get_from_api_raw(args, &x));
            } else {
                let mut res = get_from_api(args, &x);
                if !t {
                    if let CFResult::CFContestStandings(ref mut standings) = res
                    {
                        for p in &mut standings.problems {
                            let _ = p.fetch_testcases();
                        }
                    } else {
                        exit_with_error(
                            "something went wrong while parsing response",
                        );
                    }
                }
                println!("{}", res);
            }
        }
        ("status", Some(args)) => {
            let i = get_optional_arg_of_type(args, "CONTESTID").unwrap();
            let h = get_optional_arg_of_type(args, "handle");
            let f = get_optional_arg_of_type(args, "from");
            let n = get_optional_arg_of_type(args, "count");
            let x = CFContestCommand::Status {
                contest_id: i,
                handle: h,
                from: f,
                count: n,
            };
            if args.is_present("raw") {
                println!("{}", get_from_api_raw(args, &x));
            } else {
                println!("{}", get_from_api(args, &x))
            }
        }
        ("testcases", Some(args)) => {
            let i = get_optional_arg_of_type(args, "CONTESTID").unwrap();
            let t = get_optional_arg_of_type(args, "timeout").unwrap();
            let w = args.occurrences_of("wait") > 0;
            let dt = get_optional_arg_of_type(args, "wait").unwrap();
            let (k, s) = get_api_key_secret(args);

            // Check start time of contest using contest.info and searching
            // by contestId (only if --wait flag enabled)
            let time_to_start = if w {
                let list_req = CFContestCommand::List { gym: None };
                let res = list_req.get(&k, &s);
                if let Ok(CFResult::CFContestVec(ref v)) = res {
                    v.iter().find_map(|contest| {
                        if contest.id == i {
                            match contest.relative_time_seconds {
                                Some(t) => Some(std::cmp::max(0, -t)),
                                None => None,
                            }
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(t) = time_to_start {
                eprintln!("sleeping until start ({} seconds from now)", t);
                std::thread::sleep(std::time::Duration::from_secs(t as u64));
            }

            let start_instant = std::time::Instant::now();

            // Downloading testcases first requires knowing the list of problem
            // indices. These are fetched with contest.standings api req.
            let x = CFContestCommand::Standings {
                contest_id: i,
                handles: None,
                from: Some(1),
                count: Some(1),
                room: None,
                show_unofficial: None,
            };
            loop {
                eprintln!("fetching problems");
                let mut res = x.get(&k, &s);
                match res {
                    Ok(CFResult::CFContestStandings(ref mut standings)) => {
                        // Continue to fetch testcases for all problems in the
                        // contest until no errors are returned.
                        let mut done = true;
                        for p in &mut standings.problems {
                            let testcases = p.fetch_testcases();
                            done &= !w | testcases.is_ok();
                            if !done {
                                eprintln!(
                                    "failed to fetch testcases for problem {}",
                                    p.index.as_ref().unwrap()
                                );
                                break;
                            }
                        }
                        if done {
                            for p in &standings.problems {
                                println!(
                                    "--- NEW PROBLEM ---\n{}",
                                    p.index.as_ref().unwrap()
                                );
                                if let Some(ref testcases) = p.input_testcases {
                                    for t in testcases {
                                        println!(
                                            "+++ NEW TESTCASE +++\n{}\n",
                                            t
                                        );
                                    }
                                }
                            }
                            break;
                        }
                    }
                    Ok(_) => {
                        exit_with_error(
                            "Incorrectly parsed contest standings \
                            response object.",
                        );
                    }
                    Err(ApiError::CodeforcesApi(e)) => {
                        // Error returned by Codeforces api probably indicates
                        // that the aren't ready yet. Ignore and keep
                        // looping.
                        if !w {
                            eprintln!(
                                "To wait for the contest to start use \
                                the --wait flag."
                            );
                            exit_with_error(e);
                        }
                    }
                    Err(e) => {
                        exit_with_error(e);
                    }
                }
                if start_instant.elapsed().as_secs_f32() > t {
                    exit_with_error("timed out waiting for contest");
                }

                if !w {
                    break;
                }

                std::thread::sleep(std::time::Duration::from_secs_f32(dt));
                eprintln!(
                    "wasn't able to fetch (all) problems, sleeping and \
                    trying again"
                );
            }
        }
        _ => {
            exit_with_usage(matches);
        }
    }
}

pub fn problemset_command(matches: &ArgMatches) {
    match matches.subcommand() {
        ("problems", Some(args)) => {
            let t = get_optional_args_of_type(args, "tags");
            let n = get_optional_arg_of_type(args, "problemsetname");
            let f = args.is_present("dontfetchtestcases");
            let x = CFProblemsetCommand::Problems {
                tags: t,
                problemset_name: n,
            };
            if args.is_present("raw") {
                println!("{}", get_from_api_raw(args, &x));
            } else {
                let mut res = get_from_api(args, &x);
                if !f {
                    if let CFResult::CFProblemset(ref mut problemset) = res {
                        for p in &mut problemset.problems {
                            let _ = p.fetch_testcases();
                        }
                    } else {
                        exit_with_error(
                            "something went wrong while parsing response",
                        );
                    }
                }
                println!("{}", res);
            }
        }
        ("recentstatus", Some(args)) => {
            let n = get_optional_arg_of_type(args, "count").unwrap();
            let s = get_optional_arg_of_type(args, "problemsetname");
            let x = CFProblemsetCommand::RecentStatus {
                count: n,
                problemset_name: s,
            };
            if args.is_present("raw") {
                println!("{}", get_from_api_raw(args, &x));
            } else {
                println!("{}", get_from_api(args, &x))
            }
        }
        _ => {
            exit_with_usage(matches);
        }
    }
}

pub fn recentactions_command(args: &ArgMatches) {
    let s = get_optional_arg_of_type(args, "maxcount").unwrap();
    let x = CFRecentActionsCommand { max_count: s };
    if args.is_present("raw") {
        println!("{}", get_from_api_raw(args, &x));
    } else {
        println!("{}", get_from_api(args, &x))
    }
}

pub fn user_command(matches: &ArgMatches) {
    match matches.subcommand() {
        ("blogentries", Some(args)) => {
            let s = match get_optional_arg_of_type(args, "HANDLE") {
                Some(s) => s,
                None => match config::get_config() {
                    Ok(config::Config {
                        default_user: Some(u),
                        ..
                    }) => u,
                    Err(e) => exit_with_error(
                        "unable to access defaults \
                            in config.yml, specific error: \n\t"
                            .to_string()
                            + e,
                    ),
                    _ => exit_with_error(
                        "no default user set, \
                            either run `caffeine config` to do so, or provide \
                            a handle as the first argument (see help)",
                    ),
                },
            };
            let x = CFUserCommand::BlogEntries { handle: s };
            if args.is_present("raw") {
                println!("{}", get_from_api_raw(args, &x));
            } else {
                println!("{}", get_from_api(args, &x))
            }
        }
        ("friends", Some(args)) => {
            let o = args.is_present("onlyonline");
            let x = CFUserCommand::Friends {
                only_online: Some(o),
            };
            if args.is_present("raw") {
                println!("{}", get_from_api_raw(args, &x));
            } else {
                println!("{}", get_from_api(args, &x))
            }
        }
        ("info", Some(args)) => {
            let v = match get_optional_args_of_type(args, "HANDLES") {
                Some(hv) => hv,
                None => match config::get_config() {
                    Ok(config::Config {
                        default_user: Some(u),
                        ..
                    }) => vec![u],
                    Err(e) => exit_with_error(
                        "unable to access defaults \
                            in config.yml, specific error: \n\t"
                            .to_string()
                            + e,
                    ),
                    _ => exit_with_error(
                        "no default user set, \
                            either run `caffeine config` to do so, or provide \
                            handles as arguments (see help)",
                    ),
                },
            };
            let x = CFUserCommand::Info { handles: v };
            if args.is_present("raw") {
                // TODO remove `usecached` cli flag
                println!("{}", get_from_api_raw(args, &x));
            } else {
                println!("{}", get_from_api(args, &x))
            }
        }
        ("ratedlist", Some(args)) => {
            let o = args.is_present("activeonly");
            let x = CFUserCommand::RatedList {
                active_only: Some(o),
            };
            if args.is_present("raw") {
                println!("{}", get_from_api_raw(args, &x));
            } else {
                println!("{}", get_from_api(args, &x))
            }
        }
        ("rating", Some(args)) => {
            let s = match get_optional_arg_of_type(args, "HANDLE") {
                Some(s) => s,
                None => match config::get_config() {
                    Ok(config::Config {
                        default_user: Some(u),
                        ..
                    }) => u,
                    Err(e) => exit_with_error(
                        "unable to access defaults \
                            in config.yml, specific error: \n\t"
                            .to_string()
                            + e,
                    ),
                    _ => exit_with_error(
                        "no default user set, \
                            either run `caffeine config` to do so, or provide \
                            a handle as the first argument (see help)",
                    ),
                },
            };
            let x = CFUserCommand::Rating { handle: s };
            if args.is_present("raw") {
                println!("{}", get_from_api_raw(args, &x));
            } else {
                println!("{}", get_from_api(args, &x))
            }
        }
        ("status", Some(args)) => {
            let s = match get_optional_arg_of_type(args, "HANDLE") {
                Some(s) => s,
                None => match config::get_config() {
                    Ok(config::Config {
                        default_user: Some(u),
                        ..
                    }) => u,
                    Err(e) => exit_with_error(
                        "unable to access defaults \
                            in config.yml, specific error: \n\t"
                            .to_string()
                            + e,
                    ),
                    _ => exit_with_error(
                        "no default user set, \
                            either run `caffeine config` to do so, or provide \
                            a handle as the first argument (see help)",
                    ),
                },
            };
            let f = get_optional_arg_of_type(args, "from");
            let n = get_optional_arg_of_type(args, "count");
            let x = CFUserCommand::Status {
                handle: s,
                from: f,
                count: n,
            };
            if args.is_present("raw") {
                println!("{}", get_from_api_raw(args, &x));
            } else {
                println!("{}", get_from_api(args, &x))
            }
        }
        _ => {
            exit_with_usage(matches);
        }
    }
}

pub fn submit_command(args: &ArgMatches) {
    let (handle, password) = get_login_details(args);
    let c = get_optional_arg_of_type(args, "CONTESTID").unwrap();
    let p = get_optional_arg_of_type::<String>(args, "PROBLEMID").unwrap();
    let m = get_optional_arg_of_type(args, "mirror");
    let l = match get_optional_arg_of_type(args, "programtypeid") {
        Some(s) => s,
        None => match config::get_config() {
            Ok(config::Config {
                default_program_type_id: Some(id),
                ..
            }) => id,
            Err(e) => exit_with_error(
                "unable to access defaults \
                    in config.yml, specific error: \n\t"
                    .to_string()
                    + e,
            ),
            _ => exit_with_error(
                "no default program type id (programming \
                   language) set, either run `caffeine config` to do so, or \
                   provide one as an argument (see help)",
            ),
        },
    };
    let src = match submit::grab_text_from_stdin() {
        Some(s) => {
            if args.is_present("FILENAME") {
                eprintln!(
                    "FILENAME (cli option) is being ignored since text \
                    was found on stdin"
                );
            }
            s
        }
        None => {
            if args.is_present("FILENAME") {
                let filename =
                    get_optional_arg_of_type::<String>(args, "FILENAME")
                        .unwrap();
                eprintln!("loading {}", filename);
                match File::open(Path::new(&filename)) {
                    Ok(mut f) => {
                        let mut buf = String::new();
                        match f.read_to_string(&mut buf) {
                            Ok(n_bytes) => {
                                if n_bytes > 0 {
                                    buf
                                } else {
                                    exit_with_error("file empty")
                                }
                            }
                            Err(e) => exit_with_error(format!(
                                "unable to read \
                                    from file: {}",
                                e
                            )),
                        }
                    }
                    Err(e) => exit_with_error(e),
                }
            } else {
                exit_with_error(
                    "no file provided with --file or from stdin \
                    pipe",
                )
            }
        }
    };
    let res = submit::submit_from_string(&src, c, &p, &handle, &password, l, m);
    match res {
        Ok(()) => {
            eprintln!("successful submission");
        }
        Err(e) => exit_with_error(e),
    }
}
