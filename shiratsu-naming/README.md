# shiratsu-naming

[![Latest Version](https://img.shields.io/crates/v/shiratsu-naming.svg)](https://crates.io/crates/shiratsu-naming) [![Docs](https://docs.rs/shiratsu-naming/badge.svg)](https://docs.rs/shiratsu-naming) ![License](https://img.shields.io/crates/l/shiratsu-naming)

`shiratsu-naming` is a parser for catalogued ROM file names that follow common naming conventions.
It provides a zero-copy tokenizing API for file names from the following supported naming conventions.

* [No-Intro](https://datomatic.no-intro.org/stuff/The%20Official%20No-Intro%20Convention%20(20071030).pdf)
* [TOSEC](https://www.tosecdev.org/tosec-naming-convention)
* [GoodTools](https://raw.githubusercontent.com/SnowflakePowered/shiratsu/25f2c858dc3a9373e27de3df559cd00931d8e55f/shiratsu-naming/src/naming/goodtools/GoodCodes.txt)

`shiratsu-naming` does not use regular expressions and is throughly tested against a large set of names to support a variety of edge cases for each supported naming convention.

See the [crate documentation](https://docs.rs/shiratsu-naming) for usage examples and more.
