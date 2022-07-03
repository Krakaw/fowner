<p align="center">
    <img style="width: 300px" src="web/public/images/logo.svg"/>
</p>
<p>
  <img alt="Version" src="https://img.shields.io/badge/version-0.1.0-blue.svg?cacheSeconds=2592000" />
  <a href="https://twitter.com/krakaw_1" target="_blank">
    <img alt="Twitter: krakaw_1" src="https://img.shields.io/twitter/follow/krakaw_1.svg?style=social" />
  </a>
</p>

> The aptly named F-Owner is a dynamic system to track who owns a feature and when any files within that feature set are changed.
> It does this by extracting author information from the git repository and allowing features to be set via commit messages.


### 🏠 [Homepage](https://github.com/Krakaw/fowner)

[//]: # (### ✨ [Demo]&#40;krakaw.github.io/fowner&#41;)

## Install

```sh
cargo build --release
```

## Usage

```sh
cargo run -- --help
```

```
fowner 0.1.1
Krakaw <41575888+Krakaw@users.noreply.github.com>

USAGE:
    fowner [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -d, --database-path <DATABASE_PATH>    Database path [default: ./.data.sqlite3]
    -h, --help                             Print help information
    -V, --version                          Print version information

SUBCOMMANDS:
    dotfile    Generate a dotfile in the target repo containing all files and their features
    help       Print this message or the help of the given subcommand(s)
    history    Process the git history for a repository
    serve      Serve the HTTP REST API [default: 0.0.0.0:8080]
```

## Run tests

```sh
cargo test
```

## Author

👤 **Krakaw**

* Website: https://krakaw.com
* Twitter: [@krakaw_1](https://twitter.com/krakaw_1)
* Github: [@Krakaw](https://github.com/Krakaw)

## Examples

### Adding features

> Features can be added via appending `[Feature 1,Feature 2]` to pull request titles.
> 
> Or features can be added via generating a dotfile and manually adding a features to a file as a comma separated list.

### Adding Owners

> `Owners` are extracted via the import process and automatically stored against files.
> 
> Each `Owner` can have a `primary_owner_id` this solves for where Github handles have been confused so that a single owner can be presented in the results.

## Show your support

Give a ⭐️ if this project helped you!

***
_This README was generated with ❤️ by [readme-md-generator](https://github.com/kefranabg/readme-md-generator)_
