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
