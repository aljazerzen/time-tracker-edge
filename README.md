# Time Tracker EdgeDB

CLI for time-tracking using [EdgeDB](https://www.edgedb.com/).

## Installation

1. Clone the repo
2. `edgedb cloud login`
2. `edgedb project init`
3. `cargo build`

## Usage

Could not be simpler:

```
➜ tte help
Usage: tte <COMMAND>

Commands:
  start    Starts time tracker
  stop     Stops time tracker
  list     Lists all entries
  project  Manage projects
  login
  logout
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Data storage

All your entries will be stored at EdgeDB cloud.
Anyone with access to your instance will be able to read and write to the whole database.
