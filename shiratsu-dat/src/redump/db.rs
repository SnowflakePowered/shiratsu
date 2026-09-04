//! Reader for Redump's versioned SQLite database export.

use crate::error::{DatError, Result as DatResult};
use crate::{CueSheet, GameEntry, RomEntry, Serial};
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use shiratsu_naming::naming::nointro::NoIntroName;
use shiratsu_naming::naming::TokenizedName;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;
use tempfile::TempPath;
use unicode_normalization::UnicodeNormalization;

const SQLITE_HEADER: &[u8] = b"SQLite format 3\0";
const ZSTD_HEADER: &[u8] = &[0x28, 0xb5, 0x2f, 0xfd];
const SUPPORTED_SCHEMA_VERSION: i64 = 1;

/// Errors encountered while opening or reading a Redump database export.
#[derive(Debug)]
pub enum DatabaseError {
    Io(io::Error),
    Sqlite(rusqlite::Error),
    Json(serde_json::Error),
    InvalidFormat,
    UnsupportedSchemaVersion(i64),
    MissingSystem(String),
}

impl Display for DatabaseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "Redump database I/O error: {}", error),
            Self::Sqlite(error) => write!(formatter, "Redump SQLite error: {}", error),
            Self::Json(error) => write!(formatter, "Invalid JSON in Redump database: {}", error),
            Self::InvalidFormat => write!(
                formatter,
                "Redump database is neither SQLite nor Zstandard-compressed SQLite"
            ),
            Self::UnsupportedSchemaVersion(version) => write!(
                formatter,
                "Unsupported Redump database schema version {}; expected {}",
                version, SUPPORTED_SCHEMA_VERSION
            ),
            Self::MissingSystem(system) => {
                write!(formatter, "Redump database has no system named {}", system)
            }
        }
    }
}

impl StdError for DatabaseError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::Json(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for DatabaseError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

impl From<serde_json::Error> for DatabaseError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

/// A system represented in a Redump database export.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct System {
    code: String,
    name: String,
    manufacturer: String,
}

impl System {
    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn manufacturer(&self) -> &str {
        &self.manufacturer
    }
}

/// An opened Redump SQLite export.
pub struct Database {
    connection: Connection,
    // Kept after the connection so it is deleted only after SQLite closes it.
    _temporary_path: Option<TempPath>,
}

impl Database {
    /// Opens either a Redump `.sqlite` export or its distributed `.sqlite.zst` form.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let mut source = File::open(path)?;
        let mut header = [0_u8; 16];
        if let Err(error) = source.read_exact(&mut header) {
            return if error.kind() == io::ErrorKind::UnexpectedEof {
                Err(DatabaseError::InvalidFormat)
            } else {
                Err(error.into())
            };
        }
        source.seek(SeekFrom::Start(0))?;

        if header == SQLITE_HEADER {
            return Self::from_connection(Self::open_read_only(path)?, None);
        }

        if header.starts_with(ZSTD_HEADER) {
            let mut temporary = tempfile::NamedTempFile::new()?;
            let mut decoder = zstd::stream::read::Decoder::new(source)?;
            io::copy(&mut decoder, &mut temporary)?;
            temporary.flush()?;
            let temporary_path = temporary.into_temp_path();
            let connection = Self::open_read_only(&temporary_path)?;
            return Self::from_connection(connection, Some(temporary_path));
        }

        Err(DatabaseError::InvalidFormat)
    }

    fn from_connection(connection: Connection, temporary_path: Option<TempPath>) -> Result<Self> {
        let schema_version: i64 =
            connection.query_row("PRAGMA user_version", params![], |row| row.get(0))?;
        if schema_version != SUPPORTED_SCHEMA_VERSION {
            return Err(DatabaseError::UnsupportedSchemaVersion(schema_version));
        }

        let metadata_version: Option<String> = connection
            .query_row(
                "SELECT value FROM export_metadata WHERE key = 'schema_version'",
                params![],
                |row| row.get(0),
            )
            .optional()?;
        if metadata_version.as_deref() != Some("1") {
            let version = metadata_version
                .as_deref()
                .and_then(|value| value.parse().ok())
                .unwrap_or(schema_version);
            return Err(DatabaseError::UnsupportedSchemaVersion(version));
        }

        Ok(Self {
            connection,
            _temporary_path: temporary_path,
        })
    }

    /// Lists the systems which have at least one disc in the export.
    pub fn systems(&self) -> Result<Vec<System>> {
        let mut statement = self.connection.prepare(
            "SELECT s.code, s.name, s.manufacturer
             FROM systems s
             WHERE EXISTS (
                 SELECT 1 FROM discs d
                 WHERE d.system_code = s.code AND d.status <> 'Questionable'
             )
             ORDER BY s.code",
        )?;
        let systems = statement
            .query_map(params![], |row| {
                Ok(System {
                    code: row.get(0)?,
                    name: row.get(1)?,
                    manufacturer: row.get(2)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(systems)
    }

    /// Reads the active Redump entries for one Redump system code.
    ///
    /// Database and schema failures are returned by the outer result. As with
    /// the DAT parsers, naming failures are isolated to the affected entry.
    pub fn entries(&self, system_code: &str) -> Result<Vec<DatResult<GameEntry>>> {
        if !self.system_exists(system_code)? {
            return Err(DatabaseError::MissingSystem(system_code.to_string()));
        }

        let mut regions = self.regions(system_code)?;
        let mut languages = self.languages(system_code)?;
        let mut files = self.files(system_code)?;
        let mut statement = self.connection.prepare(
            "SELECT d.id, d.title, d.disc_number, d.disc_title, d.filename_suffix,
                    d.serial, d.version, d.cue, mt.rom_extension
             FROM discs d
             JOIN media_types mt ON mt.code = d.media_type_code
             WHERE d.system_code = ?1 AND d.status <> 'Questionable'
             ORDER BY LOWER(d.title), d.id",
        )?;

        let discs = statement
            .query_map(params![system_code], |row| {
                Ok(Disc {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    disc_number: row.get(2)?,
                    disc_title: row.get(3)?,
                    filename_suffix: row.get(4)?,
                    serials_json: row.get(5)?,
                    version: row.get(6)?,
                    cue: row.get(7)?,
                    rom_extension: row.get(8)?,
                    regions: vec![],
                    languages: vec![],
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        discs
            .into_iter()
            .map(|mut disc| {
                let serials: Vec<String> = serde_json::from_str(&disc.serials_json)?;
                disc.regions = regions.remove(&disc.id).unwrap_or_default();
                disc.languages = languages.remove(&disc.id).unwrap_or_default();
                let disc_files = files.remove(&disc.id).unwrap_or_default();
                Ok(GameEntry::build_entry(disc, serials, disc_files))
            })
            .collect()
    }

    fn system_exists(&self, system_code: &str) -> Result<bool> {
        let exists: i64 = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM systems WHERE code = ?1)",
            params![system_code],
            |row| row.get(0),
        )?;
        Ok(exists != 0)
    }

    fn regions(&self, system_code: &str) -> Result<HashMap<i64, Vec<String>>> {
        self.grouped_strings(
            "SELECT dr.disc_id, r.name
             FROM disc_regions dr
             JOIN regions r ON r.code = dr.region_code
             JOIN discs d ON d.id = dr.disc_id
             WHERE d.system_code = ?1 AND d.status <> 'Questionable'
             ORDER BY dr.disc_id, r.sort_order, r.code",
            system_code,
        )
    }

    fn languages(&self, system_code: &str) -> Result<HashMap<i64, Vec<String>>> {
        self.grouped_strings(
            "SELECT dl.disc_id, l.code
             FROM disc_languages dl
             JOIN languages l ON l.code = dl.language_code
             JOIN discs d ON d.id = dl.disc_id
             WHERE d.system_code = ?1 AND d.status <> 'Questionable'
             ORDER BY dl.disc_id, l.sort_order, l.code",
            system_code,
        )
    }

    fn grouped_strings(&self, sql: &str, system_code: &str) -> Result<HashMap<i64, Vec<String>>> {
        let mut statement = self.connection.prepare(sql)?;
        let rows = statement.query_map(params![system_code], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut groups = HashMap::new();
        for row in rows {
            let (disc_id, value) = row?;
            groups.entry(disc_id).or_insert_with(Vec::new).push(value);
        }
        Ok(groups)
    }

    fn files(&self, system_code: &str) -> Result<HashMap<i64, Vec<DatabaseFile>>> {
        let mut statement = self.connection.prepare(
            "SELECT f.disc_id, f.track_number, f.size, f.crc32, f.md5, f.sha1
             FROM files f
             JOIN discs d ON d.id = f.disc_id
             WHERE d.system_code = ?1 AND d.status <> 'Questionable'
             ORDER BY f.disc_id, CAST(f.track_number AS INTEGER), f.track_number",
        )?;
        let rows = statement.query_map(params![system_code], |row| {
            Ok(DatabaseFile {
                disc_id: row.get(0)?,
                track_number: row.get(1)?,
                size: row.get(2)?,
                crc: row.get(3)?,
                md5: row.get(4)?,
                sha1: row.get(5)?,
            })
        })?;
        let mut files = HashMap::new();
        for row in rows {
            let file = row?;
            files
                .entry(file.disc_id)
                .or_insert_with(Vec::new)
                .push(file);
        }
        Ok(files)
    }

    fn open_read_only(path: &Path) -> rusqlite::Result<Connection> {
        Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
    }
}

struct Disc {
    id: i64,
    title: String,
    disc_number: Option<String>,
    disc_title: Option<String>,
    filename_suffix: Option<String>,
    serials_json: String,
    version: Option<String>,
    cue: Option<String>,
    rom_extension: String,
    regions: Vec<String>,
    languages: Vec<String>,
}

struct DatabaseFile {
    disc_id: i64,
    track_number: Option<String>,
    size: i64,
    crc: String,
    md5: String,
    sha1: String,
}

struct NamedRom {
    entry: RomEntry,
    is_cue: bool,
}

impl GameEntry {
    fn build_entry(disc: Disc, serials: Vec<String>, files: Vec<DatabaseFile>) -> DatResult<Self> {
        let entry_name = EntryName::new(&disc);
        let total_tracks = files
            .iter()
            .filter(|file| file.track_number.is_some())
            .count();
        let cue_is_active = disc.rom_extension.eq_ignore_ascii_case("bin");
        let mut cue_file_name = None;
        let mut roms: Vec<NamedRom> = files
            .into_iter()
            .filter(|file| file.track_number.is_some() || cue_is_active)
            .map(|mut file| {
                let is_cue = file.track_number.is_none();
                let extension = if is_cue {
                    "cue"
                } else {
                    disc.rom_extension.as_str()
                };
                let file_name = entry_name.build_rom_name(
                    file.track_number.as_deref(),
                    total_tracks,
                    extension,
                );
                if is_cue {
                    cue_file_name = Some(file_name.clone());
                }
                file.crc.make_ascii_lowercase();
                file.md5.make_ascii_lowercase();
                file.sha1.make_ascii_lowercase();
                NamedRom {
                    entry: RomEntry {
                        md5: Some(file.md5),
                        sha1: Some(file.sha1),
                        crc: Some(file.crc),
                        file_name,
                        size: file.size,
                    },
                    is_cue,
                }
            })
            .collect();
        roms.sort_by(|left, right| {
            right.is_cue.cmp(&left.is_cue).then_with(|| {
                left.entry
                    .file_name()
                    .to_lowercase()
                    .cmp(&right.entry.file_name().to_lowercase())
            })
        });

        let cue_sheets = match (disc.cue, cue_file_name) {
            (Some(contents), Some(file_name)) if !contents.is_empty() => {
                vec![CueSheet::new(file_name, contents.into_bytes())]
            }
            (Some(contents), None) if !contents.is_empty() => {
                return Err(DatError::ParseError(format!(
                    "Redump disc {} contains a CUE sheet without a CUE file entry",
                    disc.id
                )))
            }
            _ => vec![],
        };

        let serials = serials
            .into_iter()
            .map(|serial| serial.trim().to_string())
            .filter(|serial| !serial.is_empty())
            .map(Serial::new)
            .collect();
        let mut info: crate::NameInfo = NoIntroName::try_parse(entry_name.as_ref())?.into();
        if let Some(version) = disc
            .version
            .as_deref()
            .map(str::trim)
            .filter(|version| !version.is_empty())
        {
            info.version = Some(version.to_string());
        }

        Ok(Self::new(
            entry_name.into_string(),
            Some(disc.title),
            roms.into_iter().map(|rom| rom.entry).collect(),
            serials,
            cue_sheets,
            "Redump",
            Some(info),
        ))
    }
}

struct EntryName(String);

impl EntryName {
    fn new(disc: &Disc) -> Self {
        let mut name = Self::sanitize_filename(&disc.title);
        let regions = Self::sanitized_values(&disc.regions).join(", ");
        if !regions.is_empty() {
            Self::push_parenthetical(&mut name, &regions);
        }

        let languages: Vec<String> = Self::sanitized_values(&disc.languages)
            .into_iter()
            .map(Self::capitalize_first)
            .collect();
        if languages.len() > 1 {
            Self::push_parenthetical(&mut name, &languages.join(","));
        }

        if let Some(number) = disc
            .disc_number
            .as_deref()
            .and_then(Self::sanitize_component)
        {
            Self::push_parenthetical(&mut name, &format!("Disc {}", number));
        }
        if let Some(title) = disc
            .disc_title
            .as_deref()
            .and_then(Self::sanitize_component)
        {
            Self::push_parenthetical(&mut name, &title);
        }
        if let Some(suffix) = disc
            .filename_suffix
            .as_deref()
            .and_then(Self::sanitize_component)
        {
            Self::push_parenthetical(&mut name, &suffix);
        }
        Self(name)
    }

    fn sanitized_values(values: &[String]) -> Vec<String> {
        values
            .iter()
            .filter_map(|value| Self::sanitize_component(value))
            .collect()
    }

    fn sanitize_component(value: &str) -> Option<String> {
        let value = Self::sanitize_filename(value);
        (!value.is_empty()).then_some(value)
    }

    fn capitalize_first(value: String) -> String {
        let mut characters = value.chars();
        match characters.next() {
            Some(first) => first.to_uppercase().chain(characters).collect::<String>(),
            None => value,
        }
    }

    fn push_parenthetical(name: &mut String, value: &str) {
        if !name.is_empty() {
            name.push(' ');
        }
        name.push('(');
        name.push_str(value);
        name.push(')');
    }

    fn sanitize_filename(value: &str) -> String {
        let mut value: String = value.nfc().collect();
        for (source, replacement) in [
            ("²", "^2"),
            ("³", "^3"),
            ("α", "Alpha"),
            ("½", "1-2"),
            ("Δ", "Delta"),
            ("μ", "Mu"),
            ("#", ""),
            ("¡", ""),
            ("¿", ""),
            ("°", ""),
        ] {
            value = value.replace(source, replacement);
        }
        let mut ascii = String::with_capacity(value.len());
        for character in value.chars() {
            if character.is_ascii() {
                ascii.push(character);
            } else {
                let replacement = any_ascii::any_ascii_char(character);
                if replacement.is_empty() {
                    ascii.push('-');
                } else {
                    ascii.push_str(replacement);
                }
            }
        }
        value = ascii;
        for (source, replacement) in [
            (" : ", " - "),
            (": ", " - "),
            (" / ", " & "),
            (":", "-"),
            ("/", "-"),
            ("\\", "-"),
            ("<", "_"),
            (">", "_"),
            ("\"", ""),
            ("*", "-"),
            (" ?", ""),
            ("?", ""),
            ("|", "+"),
        ] {
            value = value.replace(source, replacement);
        }
        value.retain(|character| !character.is_control());
        value
    }

    fn build_rom_name(
        &self,
        track_number: Option<&str>,
        total_tracks: usize,
        extension: &str,
    ) -> String {
        let mut name = self.0.clone();
        if total_tracks > 1 {
            if let Some(track_number) = track_number {
                let number = track_number.parse::<u32>().unwrap_or(0);
                if total_tracks >= 10 {
                    name.push_str(&format!(" (Track {:02})", number));
                } else {
                    name.push_str(&format!(" (Track {})", number));
                }
            }
        }
        name.push('.');
        name.push_str(extension);
        name
    }

    fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for EntryName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
