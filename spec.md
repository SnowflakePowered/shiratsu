# Shiragame database specification

**Schema Version:** `2.0.0`
**Stone Version:** `^9.2.0`

This document defines the schema and semantics of the Shiragame games database. 

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://tools.ietf.org/html/rfc2119).

Defined terms will be indicated in *italics* throughout the document. Their definition can be found at the end of this document, in the [section headered Definitions](#definitions).

## Preface

Shiragame is a games database similar to [OpenVGDB](https://github.com/OpenVGDB/OpenVGDB), created primarily for use with [Snowflake](https://github.com/SnowflakePowered/snowflake), but is also intended to be widely applicable to many use cases involving emulation, game preservation, archival, and verification. Shiragame uses [Stone](https://stone.snowflakepowe.red/#/) *platform ID*s and mimetypes to facilitate precise identification of the gaming platform a *dump* belongs to, as well as the format of a *dump*. 

For definitions of "*platform*" and "*format*", please refer to the Stone specification document.

### Rationale

Unlike OpenVGDB, Shiragame does not aim to be an all-in-one database. Its primary purpose is to provide an efficient method to identify and verify that a given file is a known *dump* that is part of a game distribution, and to identify the game such a *dump* is part of. Shiragame does not aim to catalogue information outside of what can be ascertained from the *game entry*'s *canonical name*. Once a searchable title is obtained from a *dump*, other tools may be used to scrape more information, such as cover arts and descriptions. Shiragame however, is only meant for the first step of identification.

## Schema and Format

Shiragame is REQUIRED to be distributed as an SQLite database with the following tables.

### The Game Entry table (`game`)

Each row of the `game` table is REQUIRED to describe a single *game entry*.

| Column              | Description                                                                                                                            | Status   |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| `game_id`           | An internal ID used to refer to the `serial` and `rom` rows related to this `game` row. This ID is unstable and MUST NOT be persisted. | REQUIRED |
| `platform_id`       | The Stone *platform ID* of the *platform* this *game entry* was intended for.                                                          | REQUIRED |
| `entry_name`        | The canonical name of the *game entry*                                                                                                 | REQUIRED |
| `release_name`      | The distribution or release name of the *game entry* that is is known as.† This is usable as a search term for scraping purposes.      | REQUIRED |
| `region`            | The region the game was released under.†                                                                                               | REQUIRED |
| `part_number`       | If this *game entry* is multi-part, or is one part of multiple discs or tapes, the part number thereof.†                               | OPTIONAL |
| `is_unlicensed`     | If this *game entry* is of an unlicensed release.†                                                                                     | REQUIRED |
| `is_demo`           | If this *game entry* is a demonstration or sample release.†                                                                            | REQUIRED |
| `version`           | If this *game entry* has a revision to an earlier released *game entry*, the revision or version number thereof.†                      | OPTIONAL |
| `status`            | The *development status* of this game.†                                                                                                | REQUIRED |
| `naming_convention` | The *naming convention* used the `entry_name` conforms to.                                                                             | REQUIRED |
| `source`            | The name of the *cataloguing organization* that provided the source data.                                                              | REQUIRED |

†as ascertained from the `entry_name`, in accordance with the *naming convention* used by the source data.

The `game_id` value MUST NOT be saved or persisted anywhere outside of a query. It SHOULD NOT be used for anything except to refer to other tables that relate
to a game entry, and SHALL NOT carry any meaning across different releases of the Shiragame database. It MUST NOT be used as a canonical identifier for a 
game entry, and MAY change without incurring API breakage across different releases of the Shiragame database. They MUST only be treated as opaque cursors by the
client consumer.

### The Dump Entry table (`rom`)

Each row of the `rom` table describes a single *dump entry* with the following schema.
| Column      | Description                                                                                                 | Status      |
| ----------- | ----------------------------------------------------------------------------------------------------------- | ----------- |
| `file_name` | The *canonical filename* assigned to this *dump* by the *cataloguing organization*                          | REQUIRED    |
| `mimetype`  | The Stone mimetype of the *format* of this file the *dump entry* refers to                                  | REQUIRED    |
| `md5`       | The MD5 hash of the file this *dump entry* refers to.                                                       | RECOMMENDED |
| `crc`       | The CRC32 hash of the file this *dump entry* refers to.                                                     | RECOMMENDED |
| `sha1`      | The SHA1 hash of the file this *dump entry* refers to.                                                      | RECOMMENDED |
| `size`      | The size of the file this *dump entry* refers to in bytes.                                                  | REQUIRED    |
| `game_id`   | Refers to the *game entry* this *dump entry* belongs to. There MUST be a row in `game` with the same value. | REQUIRED    |

One or more of `md5`, `crc`, `sha1` MUST be populated. It is RECOMMENDED, but not REQUIRED, for all three to be populated.


### The Dump Entry table (`rom`)

A *game entry* MAY have one or more *dump entries*.
Each row of the `rom` table describes a single *dump entry* with the following schema.

| Column      | Description                                                                                                 | Status      |
| ----------- | ----------------------------------------------------------------------------------------------------------- | ----------- |
| `file_name` | The *canonical filename* assigned to this *dump* by the *cataloguing organization*                          | REQUIRED    |
| `mimetype`  | The Stone mimetype of the *format* of this file the *dump entry* refers to                                  | REQUIRED    |
| `md5`       | The MD5 hash of the file this *dump entry* refers to.                                                       | RECOMMENDED |
| `crc`       | The CRC32 hash of the file this *dump entry* refers to.                                                     | RECOMMENDED |
| `sha1`      | The SHA1 hash of the file this *dump entry* refers to.                                                      | RECOMMENDED |
| `size`      | The size of the file this *dump entry* refers to in bytes.                                                  | REQUIRED    |
| `game_id`   | Refers to the *game entry* this *dump entry* belongs to. There MUST be a row in `game` with the same value. | REQUIRED    |


### The Serial Number table (`serial`)

A *game entry* MAY have zero or more serial numbers that describe it.
Each row of the `serial` table describes a serial number with the following schema.

| Column       | Description                                                                                                  | Status   |
| ------------ | ------------------------------------------------------------------------------------------------------------ | -------- |
| `serial`     | The serial number as it was published by the data source.                                                    | REQUIRED |
| `normalized` | The serial number, normalized by the **normalization rules** described below.                                | REQUIRED |
| `game_id`    | Refers to the *game entry* this serial number belongs to. There MUST be a row in `game` with the same value. | REQUIRED |

#### Normalization Rules

Serial numbers are normalized according to the following normalization rules, which are defined by a combination of a *platform ID*, a **verification pattern** that when matching the entire serial, applies the rule, and the **rewrite rule** that applies. If no normalization rule matches, the normalized serial MUST be identical to the serial number as it was published by the data source. 

| Platform ID               | Verification Pattern            | Rewrite Rule                                   | Example                       |
| ------------------------- | ------------------------------- | ---------------------------------------------- | ----------------------------- |
| `SONY_PSX` and `SONY_PS2` | `^[a-zA-Z]+[-]\d+`              | `s/^([a-zA-Z]+)[-_ ](\d+)([-_ ]\w+)*$/\1-\2/g` | `SLUS 1234-GE` to `SLUS-1234` |
| `NINTENDO_GCN`            | `^DL-DOL-([\w]{4})-[-\w\(\)]+$` | `s/^DL-DOL-([\w]{4})-[-\w\(\)]+$/\1/g`         | `DL-DOL-GC3E-0-USA` to `GC3E` |

Note that the `sed`-like expressions here are for brevity, and may be implemented differently (but equivalently) in shiratsu.

### The Shiragame meta table (`shiragame`)

Describes this release of the Shiragame database. This table MUST only contain one row, with the following schema.

| Column           | Description                                                                                                      | Status   |
| ---------------- | ---------------------------------------------------------------------------------------------------------------- | -------- |
| `shiragame`      | The string `shiragame`                                                                                           | REQUIRED |
| `schema_version` | The version of the schema used by this database.                                                                 | REQUIRED |
| `stone_version`  | The version of the [Stone definitions file][stone.dist] used by this database, for *platform IDs* and mimetypes. | REQUIRED |
| `generated`      | The time this release was created, expressed as a Unix timestamp (seconds since epoch)                           | REQUIRED |
| `release`        | A version 4 UUID that identifies this Shiragame database.                                                        | REQUIRED |
| `aggregator`     | The aggregator that generated this Shiragame database. In shiratsu's case, the string `shiratsu`                 | REQUIRED |

[stone.dist]: https://github.com/SnowflakePowered/stone/blob/master/dist/stone.dist.json

## Versioning
Both the schema and the releases of the Shiragame database itself are versioned. As of schema version `^2.0.0`, The Shiragame schema is versioned with [Semantic Versioning](https://semver.org/). 

The following changes to the schema incur an API breakage, and the MAJOR version number MUST be incremented.

* Dropping or renaming a table or column.
* The MAJOR version of the Stone definitions file used is increased.
* Any sufficiently major change to the schema as decided by the project maintainers.

The following changes to the schema do not incur an API breakage, and the MINOR version number MUST be incremented.

* Adding a new column or table.
* Changing an existing **normalization rule** for serial numbers.
* Any sufficiently small change to the schema as decided by the project maintainers.

The following changes to the schema do not incur an API breakage, and the PATCH version number MUST be incremented.

* Adding a new **normalization rule** for serial numbers.
* Any sufficiently small change to the schema as decided by the project maintainers.

As of schema 2, releases of the Shiragame database are versioned by the schema version of the released database, and the Unix timestamp of its generation, in the form `X.XXXXXX`. A release is also identified by its unique UUIDv4.

### Relationship of shiratsu version to Shiragame version
Each MAJOR.MINOR version of the shiratsu application MUST generate a Shiragame database with the same MAJOR.MINOR schema version. Effectively, this means that Shiragame schema versions are tied to the shiratsu implementation. In the case where this document and shiratsu differ, it should be considered a bug in the specification.

## Data Sources
Currently, Shiragame sources data from DATs published by three *cataloguing organizations*.

* [No-Intro](https://www.no-intro.org/)
* [Redump](http://redump.org/)
* [TOSEC](https://www.tosecdev.org/)

This MAY change in future releases of Shiragame.

### Licensing
The re-distribution of a release of the Shiragame database MUST be consistent with the licensing terms of the data sources used in the release. For all three data sources in use, re-distribution is common across the community and is not discouraged by the *cataloguing organizations* that publish the source DATs. However, since no license is clearly delimited for the re-distribution of said data, Shiragame is unable to provide a clear license for its distribution.

**However**, this specification document, and the shiratsu application are distributed under the terms of the MIT license.

## Definitions
* **platform ID** 
The Stone specified ID for a platform. See the [list of defined Stone platforms](https://stone.snowflakepowe.red/#/defs/platforms).
* **platform**
Refer to the [Stone specification on Platforms](https://stone.snowflakepowe.red/#/spec/platforms).
* **format**
 Refer to the [Stone specification on Platforms](https://stone.snowflakepowe.red/#/spec/platforms).
* **canonical filename** 
The file name given to a *dump* by a *cataloguing organization*.
* **canonical name** 
The name given to a *game entry* by a *cataloguing organization*, following a *naming convention*.
* **cataloguing organization**
A group that maintains databases of *game entries* and the *dump entries* belonging to the distribution thereof.
* **naming convention**
The method of assigning names to *game entries* followed by the *cataloguing organization* that catalogues such games. The Shiragame schema does not restrict the naming convention of files. shiratsu however knows only now to handle the following naming conventions.
  * [The TOSEC Naming Convention (2015-03-23)](https://www.tosecdev.org/tosec-naming-convention), used by the TOSEC cataloguing organization. 
  * [The Official No-Intro Convention (2007-10-30)](https://datomatic.no-intro.org/stuff/The%20Official%20No-Intro%20Convention%20(20071030).pdf), used by No-Intro and Redump cataloguing organizations, with the following amendments.
   
    * Before the `[b]` Status flag, the flag (Disc X), where X is a number from 0-9 MAY appear.
    * The (Version) flag MAY appear after the (Unl) License flag.

    These amendments are used by names given by Redump
* **dump**
Any file that is part of a *game distribution* that identifies such a file by its *dump entry*.
* **game distribution**
 The set of *dumps* that are required to execute the game as it was intended on the original *platform* it was intended for, and the title of the game thereof, as it was known or distributed under.
* **game entry**
A *game distribution* that has been verified to exist, and published by a *cataloguing organization* through a *DAT*. This corresponds to a `game` element in a DAT.
* **dump entry/ROM entry**
The hashes (one or more of CRC32, MD5, or SHA1) of a *dump* that belongs to a *game entry*. This corresponds to a `rom` element in a DAT, and is referred to in shiratsu source code as a `RomEntry`.
* **DAT**
An XML file with through which *game entries* are published by *cataloguing organizations*. A valid *DAT* MUST have the following DOCTYPE

```xml
<!DOCTYPE datafile PUBLIC "-//Logiqx//DTD ROM Management Datafile//EN" "http://www.logiqx.com/Dats/datafile.dtd">
```
* **development status**
One of `release`, `prerelease`, or `prototype`. `release` indicates that the software was made commercially, whether distributed gratis or not, in an official capacity by the publisher or developer. `prerelease` indicates that the software is in an unfinished, but mostly feature complete product, that may or may not have been intentionally released. `prototype` indicates the software is an unreleased, unfinished product that was not intentionally released in any official capacity.