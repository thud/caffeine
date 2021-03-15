# Caffeine

![caffeine logo](./caffeine.png)

Caffeine is a CLI tool which allows you to easily interact with the Codeforces
API from your terminal or from custom scripts.

## Usage
```
caffeine [FLAGS] [OPTIONS] [SUBCOMMAND]
```
Options can be listed with `caffeine --help`.

## Installation
### With `cargo` Package Manager
```
cargo install caffeine-cf
```

### Manual Compilation
To compile manually, first clone this repo.
```
git clone https://github.com/thud/caffeine
```
Then, you can build and run with `cargo run`. eg:
```
cargo r -- user info thud
```

## Description
This utility uses the
[`codeforces-api`](https://crates.io/crates/codeforces-api) crate to allow
you to interact with the
[Codeforces API](https://codeforces.com/apiHelp) from the command line or
from within a custom script.
                                                                           
### Functionality:
- Full access to the every API method provided by the Codeforces platform.
- Download testcases for any given problem.
- Submit solution to any given problem from either a file or `stdin`.
- Stores default settings in a config file.
- Stores login details in a file for easier usage.

### Authentication
As of the current version of the
[`codeforces-api`](https://crates.io/crates/codeforces-api) crate, an API
key and secret is required with every request made. Instructions to
generate these can be found [here](https://codeforces.com/apiHelp) (in the
Authorization section). To provide `caffeine` with your key/secret, you
will need to either run `caffeine login` or provide them as arguments by
using the `--key`/`-k` and `--secret`/`-s` flags (see `caffeine help`). It
is recommended that you use `cargo login` instead of CLI flags since your
key and secret may be visible in your shell command history.
                                                                           
### Submitting Solutions
Solutions are submitted by using the
[`headless_chrome`](https://crates.io/crates/headless_chrome) crate. This
requires you to have a chromium-based browser installed in order for it to
work. If your browser is not auto-detected by `headless_chrome`, then you
should try setting the `CHROME` environment variable before running. For
Brave browser, for example, you might run `export CHROME=/usr/bin/brave`.
                                                                           
Submitting solutions also requires that you provide your username and
password. You can provide these with `caffeine login` or more explicitly
with the `--handle`/`-H` and `--password`/`-p` flags.

[Docs](https://docs.rs/caffeine-cf) |
[Crate](https://crates.io/crates/caffeine-cf) |
[License](LICENSE)
