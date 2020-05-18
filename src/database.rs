use rusqlite::{backup::*, named_params, params, Connection, Result as SqliteResult};

use shiratsu_lib::{
    parse::*,
    region::Region,
    stone::{FindRomMimetype, PlatformId, StonePlatforms},
};

use std::path::Path;
use std::result;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{io, io::ErrorKind};

use uuid::Uuid;

pub struct ShiratsuDatabase {
    memory_connection: Connection,
}

#[derive(Debug)]

pub enum DatabaseError {
    IOError(io::Error),
    SqliteError(rusqlite::Error),
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(err: rusqlite::Error) -> Self {
        DatabaseError::SqliteError(err)
    }
}

impl From<io::Error> for DatabaseError {
    fn from(err: io::Error) -> Self {
        DatabaseError::IOError(err)
    }
}

type Result<T> = result::Result<T, DatabaseError>;

impl ShiratsuDatabase {
    pub fn new() -> Result<ShiratsuDatabase> {
        let mut conn = Connection::open_in_memory()?;
        create_database(&mut conn)?;
        Ok(ShiratsuDatabase {
            memory_connection: conn,
        })
    }

    pub fn add_entry(&mut self, entry: &GameEntry, platform: &PlatformId) -> Result<()> {
        insert_entry(entry, platform, &mut self.memory_connection)?;
        Ok(())
    }

    pub fn save<T: AsRef<Path>>(
        mut self,
        path: T,
        step_calback: Option<fn(_: Progress)>,
    ) -> Result<()> {
        write_meta_table(&mut self.memory_connection)?;
        let path = path.as_ref();

        if path.exists() {
            return Err(DatabaseError::IOError(io::Error::new(
                ErrorKind::AlreadyExists,
                "The specified path already exists.",
            )));
        }
        let mut target = Connection::open(path)?;
        let backup = Backup::new(&self.memory_connection, &mut target)?;
        backup.run_to_completion(5, Duration::new(0, 0), step_calback)?;
        Ok(())
    }
}

fn get_unix_time_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

fn write_meta_table(conn: &mut Connection) -> SqliteResult<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "CREATE TABLE shiragame (
        shiragame TEXT,
        schema_version TEXT,
        stone_version TEXT,
        generated TEXT,
        release TEXT,
        aggregator TEXT
    )",
        params! {},
    )?;
    tx.execute_named("INSERT INTO shiragame (shiragame, schema_version, stone_version, generated, release, aggregator)
                                        VALUES(:shiragame, :schema_version, :stone_version, :generated, :release, :aggregator)",
                    named_params! {
                        ":shiragame": "shiragame",
                        ":schema_version": "2.0.0",
                        ":stone_version": StonePlatforms::version(),
                        ":generated": get_unix_time_string(),
                        ":release": Uuid::new_v4().to_string(),
                        ":aggregator": "shiratsu"
                    })?;
    tx.commit()
}

fn create_database(conn: &mut Connection) -> SqliteResult<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "CREATE TABLE game ( 
        game_id INTEGER PRIMARY KEY,
        platform_id TEXT NOT NULL,
        entry_name TEXT NOT NULL,
        entry_title TEXT,
        release_title TEXT,
        region TEXT NOT NULL,
        part_number INTEGER,
        is_unlicensed BOOLEAN NOT NULL,
        is_demo BOOLEAN NOT NULL,
        version TEXT,
        status TEXT,
        naming_convention TEXT,
        source TEXT NOT NULL
    )",
        params![],
    )?;

    tx.execute(
        "CREATE TABLE serial ( 
        serial TEXT NOT NULL,
        normalized TEXT NOT NULL,
        game_id INTEGER NOT NULL,
        FOREIGN KEY (game_id) REFERENCES game (game_id)
    )",
        params![],
    )?;

    tx.execute(
        "CREATE TABLE rom ( 
        file_name TEXT NOT NULL,
        mimetype TEXT,
        md5 TEXT,
        crc TEXT,
        sha1 TEXT,
        size INTEGER NOT NULL,
        game_id INTEGER NOT NULL,
        FOREIGN KEY (game_id) REFERENCES game (game_id)
    )",
        params![],
    )?;
    tx.commit()
}

fn insert_entry(
    entry: &GameEntry,
    platform: &PlatformId,
    conn: &mut Connection,
) -> SqliteResult<()> {
    let tx = conn.transaction()?;

    let region_str = entry
        .info()
        .map(|n| n.region())
        .map(|r| Region::to_region_string(r));

    tx.execute_named(r#"
        INSERT INTO game (
            platform_id,
            entry_name,
            entry_title,
            release_title,
            region,
            part_number,
            is_unlicensed,
            is_demo,
            version,
            status,
            naming_convention,
            source
        )
        VALUES (:platform_id, :entry_name, :entry_title, :release_title, :region, :part_number, :is_unlicensed, :is_demo, :version, :status, :naming_convention, :source)
    "#,
    named_params! {
        ":platform_id": platform.as_ref(),
        ":entry_name": entry.entry_name(),
        ":entry_title": entry.info().map(|n| n.entry_title()),
        ":release_title": entry.info().map(|n| n.release_title()),
        ":region": region_str.as_deref().unwrap_or(Region::Unknown.as_ref()),
        ":part_number": entry.info().map(|n| n.part_number()),
        ":is_unlicensed": entry.info().map(|n| n.is_unlicensed()).unwrap_or(false),
        ":is_demo": entry.info().map(|n| n.is_demo()).unwrap_or(false),
        ":version": entry.info().map(|n| n.version()),
        ":status": entry.info().map(|n| n.development_status()).unwrap_or(DevelopmentStatus::Release).as_ref(),
        ":naming_convention": entry.info().map(|n| n.naming_convention()).unwrap_or(NamingConvention::Unknown).as_ref(),
        ":source": entry.source()
    })?;

    let game_id = tx.last_insert_rowid();
    for rom in entry.rom_entries().iter() {
        tx.execute_named(
            r#"
            INSERT INTO rom(
                file_name,
                mimetype,
                md5,
                crc,
                sha1,
                size,
                game_id
            )
            VALUES (:file_name, :mimetype, :md5, :crc, :sha1, :size, :game_id)
        "#,
            named_params! {
                ":file_name": rom.file_name(),
                ":mimetype": rom.find_mimetype(platform),
                ":md5": rom.hash_md5(),
                ":crc": rom.hash_crc(),
                ":sha1": rom.hash_sha1(),
                ":size": rom.size(),
                ":game_id": game_id,
            },
        )?;
    }

    for serial in entry.serials().iter() {
        tx.execute_named(
            r#"
            INSERT INTO serial(
                serial,
                normalized,
                game_id
            )
            VALUES (:serial, :normalized, :game_id)
        "#,
            named_params! {
                ":serial" : serial.as_ref(),
                ":normalized" : serial.as_normalized(platform).as_ref().as_ref(),
                ":game_id": game_id,
            },
        )?;
    }

    tx.commit()
}
