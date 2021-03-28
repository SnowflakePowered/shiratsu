# 調ツー &bullet; shirats&#363;

Second-generation aggregator for [shiragame 2.0](https://github.com/SnowflakePowered/shiragame) databases.

**The Shiragame games database is [available here.](https://github.com/SnowflakePowered/shiragame)**

## Installation
shiratsu does not come with a binary release. You can download a source tarball and [build from source](#Building), or install from cargo.


```bash
$ cargo install shiratsu
```

## Usage

1. Add your DATs to the `unsorted` folder. You may provide your own `sortrules.yml` as needed, or shiratsu will use its internal sorting rules. Sorting rules are provided as [Unix-like globs](https://docs.rs/glob/0.3.0/glob/).
   ```bash
   $ mkdir unsorted
   $ unzip "No-Intro Love Pack (Standard) (*).zip" -d unsorted
   ``` 
2. Sort your DATs by running `sort`
   ```bash
   $ shiratsu sort
   ``` 
3. Create the database
    ```bash
    $ shiratsu database.db
    ```
    This will write the database to file, and a log file that certifies the contents of the database.

## Building

This is a pure Rust application with no external compilation dependencies besides Cargo and rustc. Simply clone the repository, and run

```bash
cargo build
```