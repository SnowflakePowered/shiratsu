# 調ツー shiratsu

Second-generation aggregator for [shiragame 2.0](https://github.com/SnowflakePowered/shiragame) databases.

**The Shiragame games database is [available here.](https://github.com/SnowflakePowered/shiragame)**

## Usage

1. Create the folder structure
   ```bash
   $ shiratsu makefolders
   ``` 
2. Add DAT in the respective folder for their platform. Currently only supports TOSEC, No-Intro, and Redump dats.
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