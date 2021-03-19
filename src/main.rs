mod database;
mod ingest;
mod log;
mod sortrules;
mod error;

use shiratsu_stone::{
    PlatformId, StonePlatforms
};

use shiratsu_parse::{
    error::*,
    dat::*,
    dat::{nointro::*, redump::*, tosec::*},
};

use anyhow::{anyhow, Error, Result};

use slog::{o, Drain, Logger};

use std::borrow::Cow;
use std::env;
use std::fs::{create_dir, File, OpenOptions};
use std::io::{self, BufRead, BufReader, ErrorKind, Seek, SeekFrom};
use std::path::Path;
use std::{ffi::OsStr, time::Instant};

use database::{DatabaseError, ShiratsuDatabase};

use console::style;
use indicatif::ProgressBar;

use lazy_static_include::*;

use glob::glob_with;
use glob::MatchOptions;
use shiratsu_parse::naming::{ToNameInfo, NamingConvention};

type ParseResult<T> = std::result::Result<T, ParseError>;

fn get_entries<R: BufRead + Seek>(
    mut reader: R,
) -> Result<Option<(Vec<ParseResult<GameEntry>>, &'static str)>> {
    reader.seek(SeekFrom::Start(0))?;
    match GameEntry::try_from_nointro_buf(reader.by_ref()) {
        Ok(entries) => return Ok(Some((entries, "No-Intro"))),
        Err(ParseError::HeaderMismatchError(_, _)) => {}
        Err(err) => return Err(Error::new(err)),
    }
    reader.seek(SeekFrom::Start(0))?;
    match GameEntry::try_from_redump_buf(reader.by_ref()) {
        Ok(entries) => return Ok(Some((entries, "Redump"))),
        Err(ParseError::HeaderMismatchError(_, _)) => {}
        Err(err) => return Err(Error::new(err)),
    }
    reader.seek(SeekFrom::Start(0))?;
    match GameEntry::try_from_tosec_buf(reader.by_ref()) {
        Ok(entries) => return Ok(Some((entries, "TOSEC"))),
        Err(ParseError::HeaderMismatchError(_, _)) => {}
        Err(err) => return Err(Error::new(err)),
    }
    Err(anyhow!("Did not match any known cataloguing organization."))
}

fn setup_logging<T: AsRef<Path>>(log_path: T, file_log_path: T) -> (slog::Logger, slog::Logger) {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .unwrap();

    let file_decorator = slog_term::PlainDecorator::new(file);

    let file_drain = slog_term::CompactFormat::new(file_decorator).build().fuse();
    let file_drain = slog_async::Async::new(file_drain)
        .chan_size(2048)
        .overflow_strategy(slog_async::OverflowStrategy::Block)
        .build()
        .fuse();

    // let drain = Duplicate(term_drain, file_drain).fuse();
    let logger = slog::Logger::root(file_drain, o!());

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file_log_path)
        .unwrap();
    let file_decorator = slog_term::PlainDecorator::new(file);

    let file_drain = slog_term::CompactFormat::new(file_decorator).build().fuse();
    let file_drain = slog_async::Async::new(file_drain)
        .chan_size(2048)
        .overflow_strategy(slog_async::OverflowStrategy::Block)
        .build()
        .fuse();
    let file_logger = slog::Logger::root(file_drain, o!());
    (logger, file_logger)
}

lazy_static_include_str!(SORTING_RULES, "sortrules.yml");

pub enum Event<'a> {
    CreatingFolderStructure,
    CreatedDirectory(&'a Path),
    DirectoryAlreadyExists(&'a Path),
    CreatedUnsortedDirectory(&'a Path),
    UnsortedDirectoryAlreadyExists(&'a Path),
    CreateFoldersSuccess,
    DatabaseSavePathAlreadyExists(&'a Path),
    GeneratingDatabase(&'a Path, &'a Logger),
    FoundDatFile(
        &'a ProgressBar,
        &'a Path,
        u64,
        &'a PlatformId,
        &'a str,
        &'a Logger,
        &'a Logger,
    ),
    ProcessEntry(
        &'a ProgressBar,
        &'a PlatformId,
        &'a Path,
        &'a str,
        &'a Logger,
    ),
    ParseEntryError(&'a ParseError, &'a Logger),
    ProcessEntrySuccess(&'a ProgressBar),
    DatProcessingSuccess(&'a ProgressBar, &'a PlatformId, &'a Path, usize, &'a Logger),
    DbSaveSuccess(&'a Path, &'a String, &'a String, u64),
    DbSaveError(&'a Path, &'a Logger),
    LoadInternalSortingRules,
    LoadExternalSortingRules,
    LoadedSortingRules(&'a str),
    SortedFile(&'a std::ffi::OsStr, &'a PlatformId),
    SortingSuccess(usize, u64),
    NoEntriesFound(&'a OsStr, &'a Logger),
}

fn create_folders<F>(event_fn: F) -> Result<()>
where
    F: Fn(Event) -> (),
{
    let mut current_dir = env::current_dir()?;
    current_dir.push("dats");
    event_fn(Event::CreatingFolderStructure);
    if !current_dir.exists() {
        create_dir(&current_dir)?;
        event_fn(Event::CreatedDirectory(&current_dir));
    }
    for platform_id in StonePlatforms::get().ids() {
        current_dir.push(platform_id.as_ref());
        if !current_dir.exists() {
            create_dir(&current_dir)?;
            event_fn(Event::CreatedDirectory(&current_dir));
        } else {
            event_fn(Event::DirectoryAlreadyExists(&current_dir));
        }
        current_dir.pop();
    }
    current_dir.pop();
    current_dir.push("unsorted");
    if !current_dir.exists() {
        create_dir(&current_dir)?;
        event_fn(Event::CreatedUnsortedDirectory(&current_dir));
    } else {
        event_fn(Event::UnsortedDirectoryAlreadyExists(&current_dir));
    }
    event_fn(Event::CreateFoldersSuccess);

    Ok(())
}

fn create_db<S: AsRef<str>, F>(save_path: S, event_fn: F) -> Result<()>
where
    F: Fn(Event) -> (),
{
    let now = Instant::now();
    let save_path = Path::new(save_path.as_ref());
    if save_path.exists() {
        event_fn(Event::DatabaseSavePathAlreadyExists(&save_path));
        return Err(anyhow::Error::new(io::Error::new(
            ErrorKind::AlreadyExists,
            "The specified path already exists.",
        )));
    }
    let (root, filelog) = setup_logging(format!("{}.log", save_path.display()), format!("{}.inputs.log", save_path.display()));
    event_fn(Event::GeneratingDatabase(&save_path, &root));

    let mut db = ShiratsuDatabase::new().unwrap();
    for (platform_id, dir) in ingest::get_paths("dats").into_iter() {
        let mut parse_errors = Vec::new();
        let reader = BufReader::new(File::open(dir.path())?);
        match get_entries(reader) {
            Ok(Some((entries, source))) => {
                let pb = ProgressBar::new(entries.len() as u64);
                event_fn(Event::FoundDatFile(
                    &pb,
                    dir.path(),
                    entries.len() as u64,
                    platform_id,
                    source,
                    &root,
                    &filelog,
                ));

                for game in entries.iter() {
                    match game {
                        Ok(game) => {
                            event_fn(Event::ProcessEntry(
                                &pb,
                                platform_id,
                                dir.path(),
                                game.entry_name(),
                                &root,
                            ));
                            db.add_entry(game, platform_id).unwrap();
                            event_fn(Event::ProcessEntrySuccess(&pb));
                        }
                        Err(err) => parse_errors.push(Event::ParseEntryError(err, &root)),
                    }
                }

                event_fn(Event::DatProcessingSuccess(
                    &pb,
                    platform_id,
                    dir.path(),
                    entries.len(),
                    &root,
                ));

                for error in parse_errors.into_iter() {
                    event_fn(error);
                }
            }
            Ok(None) => event_fn(Event::NoEntriesFound(dir.file_name(), &root)),
            Err(err) => return Err(err),
        }
    }

    match db.save(save_path, Some(log::process_duration)) {
        Ok((uuid, time)) => {
            event_fn(Event::DbSaveSuccess(
                &save_path,
                &uuid,
                &time,
                now.elapsed().as_secs(),
            ));
            Ok(())
        }
        Err(err) => {
            event_fn(Event::DbSaveError(&save_path, &root));

            Err(match err {
                DatabaseError::IOError(err) => anyhow::Error::new(err),
                DatabaseError::SqliteError(err) => anyhow::Error::new(err),
            })
        }
    }
}

fn compare<F>(event_fn: F) -> Result<()>
    where
        F: Fn(Event) -> (),
{
    let (root, filelog) = setup_logging("root.log",
                                        "file.log");

    for (platform_id, dir) in ingest::get_paths("dats").into_iter() {
        let mut parse_errors = Vec::new();
        let reader = BufReader::new(File::open(dir.path())?);
        match get_entries(reader) {
            Ok(Some((entries, source))) => {
                let pb = ProgressBar::new(entries.len() as u64);
                event_fn(Event::FoundDatFile(
                    &pb,
                    dir.path(),
                    entries.len() as u64,
                    platform_id,
                    source,
                    &root,
                    &filelog,
                ));

                for game in entries.iter() {
                    match game {
                        Ok(game) => {
                            event_fn(Event::ProcessEntry(
                                &pb,
                                platform_id,
                                dir.path(),
                                game.entry_name(),
                                &root,
                            ));

                            let old_name = game.entry_name();
                            if let Ok(res) = shiratsu_parse::naming::tosec::try_parse(old_name)
                            {
                                if res.has_warnings() {
                                    eprintln!("{}", old_name);
                                    eprintln!("{:?}", res);
                                }
                            }
                        }
                        Err(ParseError::BadFileNameError(NamingConvention::TOSEC, name)) => {
                            if let Ok(res) = shiratsu_parse::naming::tosec::try_parse(name)
                            {
                                eprintln!("NAMERRROR=====");
                                if res.has_warnings() {
                                    eprintln!("{}", name);
                                    eprintln!("{:?}", res);
                                }
                            }
                        }
                        Err(err) => parse_errors.push(Event::ParseEntryError(err, &root)),
                    }

                    event_fn(Event::ProcessEntrySuccess(&pb));
                }

                event_fn(Event::DatProcessingSuccess(
                    &pb,
                    platform_id,
                    dir.path(),
                    entries.len(),
                    &root,
                ));

                for error in parse_errors.into_iter() {
                    event_fn(error);
                }
            }
            Ok(None) => event_fn(Event::NoEntriesFound(dir.file_name(), &root)),
            Err(err) => return Err(err),
        }
    }
    Ok(())
}


fn sort_dats<F>(event_fn: F) -> Result<()>
where
    F: Fn(Event) -> (),
{
    let now = Instant::now();
    let rules = std::fs::read_to_string("sortrules.yml")
        .map_or_else(|_| Cow::Borrowed(*SORTING_RULES), |s| Cow::Owned(s));
    let sort_rules_src;

    match &rules {
        Cow::Borrowed(_) => {
            sort_rules_src = "internal sorting rules";
            event_fn(Event::LoadInternalSortingRules);
        }
        Cow::Owned(_) => {
            sort_rules_src = "sortrules.yml";
            event_fn(Event::LoadExternalSortingRules);
        }
    }

    let rules = sortrules::load_map(rules)?;

    event_fn(Event::LoadedSortingRules(&sort_rules_src));

    create_folders(&event_fn)?;
    let mut current_dir = env::current_dir()?;
    current_dir.push("dats");

    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    let mut count: usize = 0;

    for (platform_id, rules) in rules.iter() {
        current_dir.push(platform_id.as_ref());
        for path in rules
            .iter()
            .flat_map(|rule| glob_with(rule, options))
            .flat_map(|rule| rule)
            .flat_map(|result| result)
        {
            if let Some(filename) = path.file_name() {
                let path = std::fs::canonicalize(&path)?;
                current_dir.push(filename);
                std::fs::rename(&path, &current_dir)?;
                event_fn(Event::SortedFile(filename, platform_id));
                count += 1;
                current_dir.pop();
            }
        }
        current_dir.pop();
    }
    event_fn(Event::SortingSuccess(count, now.elapsed().as_secs()));
    Ok(())
}

fn run_app<F>(event_fn: F) -> Result<()>
where
    F: Fn(Event) -> (),
{
    let args = env::args().skip(1).take(1).next();
    let command = args.ok_or(io::Error::new(
        ErrorKind::NotFound,
        "No save path was specified.",
    ))?;

    match command.as_str() {
        "sort" => sort_dats(event_fn),
        "compare" => compare(event_fn),
        save_path => create_db(save_path, event_fn),
    }
}

fn main() {
    std::process::exit(match run_app(log::print_event) {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("{} -- {}", style(" ✘ Error").red(), err);
            1
        }
    });
}
