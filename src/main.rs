//! caffeine-cf bin crate - CLI tool for easily interacting with the
//! [Codeforces](https://codeforces.com/) API.
//!
//! ![caffeine logo](https://github.com/thud/caffeine/raw/master/caffeine.png)
//!
//! This utility uses the
//! [`codeforces-api`](https://crates.io/crates/codeforces-api) crate to allow
//! you to interact with the 
//! [Codeforces API](https://codeforces.com/apiHelp) from the command line or
//! from within a custom script.
//!
//! ### Authentication
//! As of the current version of the
//! [`codeforces-api`](https://crates.io/crates/codeforces-api) crate, an API
//! key and secret is required with every request made. Instructions to
//! generate these can be found [here](https://codeforces.com/apiHelp) (in the
//! Authorization section). To provide `caffeine` with your key/secret, you
//! will need to either run `caffeine login` or provide them as arguments by
//! using the `--key`/`-k` and `--secret`/`-s` flags (see `caffeine help`).
//!
//! ### Functionality:
//! - Full access to the every API method provided by the Codeforces platform.
//! - Download testcases for any given problem.
//! - Submit solution to any given problem from either a file or `stdin`.
//! - Stores default settings in a config file.
//! - Stores login details in a file for easier usage.
//!
//! ### Submitting Solutions
//! Solutions are submitted by using the
//! [`headless_chrome`](https://crates.io/crates/headless_chrome) crate. This
//! requires you to have a chromium-based browser installed in order for it to
//! work. If your browser is not auto-detected by `headless_chrome`, then you
//! should try setting the `CHROME` environment variable before running. For
//! Brave browser, for example, you might run `export CHROME=/usr/bin/brave`.
//!
//! Submitting solutions also requires that you provide your username and
//! password. You can provide these with `caffeine login` or more explicitly
//! with the `--handle`/`-H` and `--password`/`-p` flags.
//!

use clap::{crate_version, App, Arg};
mod auth;
mod config;
mod handlers;
mod submit;

pub const NAME_QUL: &str = "dev";
pub const NAME_ORG: &str = "thud";
pub const NAME_BIN: &str = "caffeine";

pub const AUTH_FILE_NAME: &str = "auth.yml";
pub const AUTH_HELP_MSG: &str = "To generate an API key & secret, go to \
                                 https://codeforces.com/settings/api";

pub const CONF_FILE_NAME: &str = "config.yml";
pub const PROGRAM_TYPE_ID_HELP: &str = "43 GNU GCC C11 5.1.0
52 Clang++17 Diagnostics
42 GNU G++11 5.1.0
50 GNU G++14 6.4.0
54 GNU G++17 7.3.0
2  Microsoft Visual C++ 2010
59 Microsoft Visual C++ 2017
61 GNU G++17 9.2.0 (64 bit, msys 2)
65 C# 8, .NET Core 3.1
9  C# Mono 6.8
28 D DMD32 v2.091.0
32 Go 1.15.6
12 Haskell GHC 8.10.1
60 Java 11.0.6
36 Java 1.8.0_241
48 Kotlin 1.4.0
19 OCaml 4.02.1
3  Delphi 7
4  Free Pascal 3.0.2
51 PascalABC.NET 3.4.2
13 Perl 5.20.1
6  PHP 7.2.13
7  Python 2.7.18
31 Python 3.9.1
40 PyPy 2.7 (7.3.0)
41 PyPy 3.7 (7.3.0)
67 Ruby 3.0.0
49 Rust 1.49.0
20 Scala 2.12.8
34 JavaScript V8 4.8.0
55 Node.js 12.6.3";

fn main() {
    let app = App::new("caffeine")
    .version(crate_version!())
    .about("A CLI tool for accessing codeforces resources")
    .author("thud <thud.dev>")
        .version_short("v")
    .args(&[
                Arg::with_name("raw")
                        .help("Boolean Flag, return raw JSON")
                        .short("r")
                        .long("raw")
                        .display_order(1000)
                        .global(true),
                Arg::with_name("key")
                        .help("String value, manually provide API key (not \
                        recommended)")
                        .requires("secret")
                        .short("k")
                        .long("key")
                        .takes_value(true)
                        .display_order(1000)
                        .global(true),
                Arg::with_name("secret")
                        .help("String value, manually provide API secret (not \
                        recommended)")
                        .requires("key")
                        .short("s")
                        .long("secret")
                        .takes_value(true)
                        .display_order(1000)
                        .global(true),
        ])
        .subcommand(
            App::new("blogentry")
            .about("Get information about a blog entry")
            .subcommand(
                App::new("comments")
                .about("Returns a list of comments on a specified blog entry")
                .args(&[
                    Arg::with_name("BLOGENTRYID")
                    .help("blogEntryId of specified blog entry")
                    .index(1)
                    .required(true)
                ])
            )
            .subcommand(
                App::new("view")
                .about("Returns specified blog entry")
                .args(&[
                    Arg::with_name("BLOGENTRYID")
                    .help("blogEntryId of specified blog entry (eg. 82347)")
                    .index(1)
                    .required(true)
                ])
            )
        )
        .subcommand(
            App::new("contest")
            .about("Get information about one or more contests")
            .subcommand(
                App::new("hacks")
                .about("Returns list of hacks in the specified contests")
                .args(&[
                    Arg::with_name("CONTESTID")
                    .help("contestId of specified contest (eg. 1466)")
                    .index(1)
                    .required(true)
                ])
            )
            .subcommand(
                App::new("list")
                .about("Returns information about all available contests")
                .args(&[
                    Arg::with_name("gym")
                    .help("If true, gym contests are returned")
                    .long("gym")
                    .short("g")
                ])
            )
            .subcommand(
                App::new("ratingchanges")
                .about("Returns rating changes after a contest")
                .args(&[
                    Arg::with_name("CONTESTID")
                    .help("contestId of specified contest (eg. 1466)")
                    .index(1)
                    .required(true)
                ])
            )
            .subcommand(
                App::new("standings")
                .about("Returns the description of the contest and the \
                    requested part of the standings")
                .args(&[
                    Arg::with_name("CONTESTID")
                    .help("contestId of specified contest (eg. 1466)")
                    .index(1)
                    .required(true),
                    Arg::with_name("from")
                    .help("1-based index of the standings row to start the \
                        ranklist")
                    .long("from")
                    .short("f")
                    .takes_value(true),
                    Arg::with_name("count")
                    .help("Number of standing rows to return")
                    .long("count")
                    .short("n")
                    .takes_value(true),
                    Arg::with_name("handles")
                    .help("List of handles")
                    .long("handles")
                    .short("H")
                    .takes_value(true)
                    .multiple(true)
                    .use_delimiter(true),
                    Arg::with_name("room")
                    .help("If specified, show only participants in given room")
                    .long("room")
                    .short("R")
                    .takes_value(true),
                    Arg::with_name("showunofficial")
                    .help("If set, all participants (virtual, out of \
                    competition) are shown")
                    .long("showunofficial")
                    .short("u"),
                    Arg::with_name("dontfetchtestcases")
                    .help("If set, don't fetch testcases for each problem (raw \
                    never fetches)")
                    .long("dontfetchtestcases")
                    .short("F"),
                ])
            )
            .subcommand(
                App::new("status")
                .about("Returns submissions for specified contest")
                .args(&[
                    Arg::with_name("CONTESTID")
                    .help("contestId of specified contest (eg. 1466)")
                    .index(1)
                    .required(true),
                    Arg::with_name("handle")
                    .help("Codeforces user handle")
                    .long("handle")
                    .short("H")
                    .takes_value(true),
                    Arg::with_name("from")
                    .help("1-based index of the first submission to return")
                    .long("from")
                    .short("f")
                    .takes_value(true),
                    Arg::with_name("count")
                    .help("Number of returned submissions")
                    .short("n")
                    .takes_value(true),
                ])
            )
            .subcommand(
                App::new("testcases")
                .about("Returns testcases for every problem (delimited) in \
                    specified contest")
                .args(&[
                    Arg::with_name("CONTESTID")
                    .help("contestId of specified contest (eg. 1466)")
                    .index(1)
                    .required(true),
                    Arg::with_name("wait")
                    .help("Flag or Number value indicating whether to wait for \
                    contest to start. (default value = 10 seconds)")
                    .long("wait")
                    .short("w")
                    .default_value("10")
                    .takes_value(true)
                    .required(true),
                    Arg::with_name("timeout")
                    .help("Program timeout value (default 1000 seconds). Only \
                        used if wait enabled.")
                    .long("timeout")
                    .short("t")
                    .takes_value(true)
                    .default_value("1000")
                    .required(true),
                ])
            )
        )
        .subcommand(
            App::new("problemset")
            .about("Get information about one or more problems")
            .subcommand(
                App::new("problems")
                .about("Returns all problems from problemset")
                .args(&[
                    Arg::with_name("tags")
                    .help("List of tags for a problemset")
                    .long("tags")
                    .short("t")
                    .takes_value(true)
                    .multiple(true)
                    .use_delimiter(true),
                    Arg::with_name("problemsetname")
                    .help("Custom problemset's short name, like `acmsguru`")
                    .long("problemsetname")
                    .short("N")
                    .takes_value(true),
                    Arg::with_name("dontfetchtestcases") // NEW
                    .help("If set, don't fetch testcases for each problem (raw \
                    never fetches)")
                    .long("dontfetchtestcases")
                    .short("F"),
                ])
            )
            .subcommand(
                App::new("recentstatus")
                .about("Returns recent submissions")
                .args(&[
                    Arg::with_name("count")
                    .long("count")
                    .short("n")
                    .takes_value(true)
                    .required(true),
                    Arg::with_name("problemsetname")
                    .long("problemsetname")
                    .short("N")
                    .takes_value(true),
                ])
            )
        )
        .subcommand(
            App::new("recentactions")
            .about("Returns recent actions")
            .args(&[
                Arg::with_name("maxcount")
                .long("maxcount")
                .short("n")
                .takes_value(true)
                .required(true),
            ])
        )
        .subcommand(
            App::new("user")
            .about("Get information about a user")
            .subcommand(
                App::new("blogentries")
                .about("Returns a list of user's blog entries")
                .args(&[
                    Arg::with_name("HANDLE")
                    .help("Codeforces user handle (set the default user \
                    with `caffeine config`) ")
                    .index(1),
                ]),
            )
            .subcommand(
                App::new("friends")
                .about("Return's friends of the currently logged in user (api \
                keys required)")
                .args(&[
                    Arg::with_name("onlyonline")
                    .help("Only return online friends")
                    .long("onlyonline")
                    .short("o"),
                ])
            )
            .subcommand(
                App::new("info")
                .about("Returns information about one or more users")
                .args(&[
                    Arg::with_name("HANDLES")
                    .help("list of handles to get info for (set the default \
                    user with `caffeine config`)")
                    .index(1)
                    .multiple(true)
                    .use_delimiter(true),
                ])
            )
            .subcommand(
                App::new("ratedlist")
                .about("Returns the list of users who have participated in \
                    >=1 rated contests")
                .args(&[
                    Arg::with_name("activeonly")
                    .help("only show users who participated in contests \
                        recently (last month)")
                    .long("activeonly")
                    .short("a"),
                ])
            )
            .subcommand(
                App::new("rating")
                .about("Returns the rating history of a specified user")
                .args(&[
                    Arg::with_name("HANDLE")
                    .help("Codeforces user handle (set the default \
                    user with `caffeine config`)")
                    .index(1)
                ])
            )
            .subcommand(
                App::new("status")
                .about("Returns submissions of a specified user")
                .args(&[
                    Arg::with_name("HANDLE")
                    .help("Codeforces user handle (set the default \
                    user with `caffeine config`)")
                    .index(1)
                    .takes_value(true),
                    Arg::with_name("from")
                    .help("1-based index of the first submission to return \
                        (recent first)")
                    .long("from")
                    .short("f")
                    .takes_value(true),
                    Arg::with_name("count")
                    .help("Number of returned submissions")
                    .long("count")
                    .short("n")
                    .takes_value(true),
                ])
            )
        )
        .subcommand(
            App::new("submit")
            .about("Submit code to a specified problem")
            .args(&[
                Arg::with_name("handle")
                .help("Handle or email to login with (not recommended)")
                .long("handle")
                .short("H")
                .takes_value(true),
                Arg::with_name("password")
                .help("Handle or email to login with (not recommended)")
                .long("password")
                .short("p")
                .takes_value(true),
                Arg::with_name("CONTESTID")
                .help("contestId of requested problem")
                .index(1)
                .required(true)
                .takes_value(true),
                Arg::with_name("PROBLEMID")
                .help("problemId of requested problem (eg A)")
                .index(2)
                .required(true)
                .takes_value(true),
                Arg::with_name("programtypeid")
                .help("programTypeId of solution to be submitted (eg 54 = \
                C++17)")
                .long("programtypeid")
                .short("l")
                .takes_value(true),
                Arg::with_name("mirror")
                .help("Number (0-3) indicating the mirror which should be used \
                (0 = codeforces.com, 1 = m1.codeforces.com ...)")
                .long("mirror")
                .short("m")
                .takes_value(true),
                Arg::with_name("FILENAME")
                .help("String value, filename of solution to be submitted \
                    (alternatively use piped stdin)")
                .index(3)
                .takes_value(true),
            ])
        )
        .subcommand(
            App::new("login")
            .about("Save api keys and login details (recommended over cli \
            flags)")
            .args(&[
                Arg::with_name("handle")
                .help("String value, provide login handle/email as cli \
                    argument (not recommended)")
                .long("handle")
                .short("H")
                .requires("password")
                .takes_value(true),
                Arg::with_name("password")
                .help("String value, provide login password as cli argument \
                    (not recommended)")
                .long("password")
                .short("p")
                .requires("handle")
                .takes_value(true),
            ])
        )
        .subcommand(
            App::new("config")
            .about("change configuration (eg. default programming language)")
            .args(&[
                Arg::with_name("defaultuser")
                .help("String value, provide default user handle for \
                    commands like `user status` as cli argument")
                .long("defaultuser")
                .short("u")
                .takes_value(true),
                Arg::with_name("defaultprogramtypeid")
                .help("Number, set default programTypeId language for \
                    submitting code (eg. 54 = C++17) as cli argument")
                .long("lang")
                .short("l")
                .takes_value(true),
            ])
        );

    let matches = app.get_matches();

    match matches.subcommand() {
        ("blogentry", Some(subcommand)) => {
            handlers::blogentry_command(subcommand);
        }
        ("contest", Some(subcommand)) => {
            handlers::contest_command(subcommand);
        }
        ("problemset", Some(subcommand)) => {
            handlers::problemset_command(subcommand);
        }
        ("recentactions", Some(args)) => {
            handlers::recentactions_command(args);
        }
        ("user", Some(subcommand)) => {
            handlers::user_command(subcommand);
        }
        ("submit", Some(args)) => {
            handlers::submit_command(args);
        }
        ("login", Some(args)) => {
            handlers::login_command(args);
        }
        ("config", Some(args)) => {
            handlers::config_command(args);
        }
        _ => {
            handlers::exit_with_usage(&matches);
        }
    }
}
