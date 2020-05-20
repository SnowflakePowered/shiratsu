use shiratsu_lib::{
    error::ShiratsuError,
    parse::*,
    parse::{nointro::*, redump::*, tosec::*},
};
use slog::{error, info, o, Drain};

use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, ErrorKind, Seek, SeekFrom};
use std::path::Path;
use std::result::Result;

mod database;
mod ingest;
use database::{DatabaseError, ShiratsuDatabase};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use lazy_static::lazy_static;
use rusqlite::backup::Progress;

#[derive(Debug)]
enum AppError {
    IOError(std::io::Error),
    ShiratsuError(ShiratsuError),
    DatabaseError(DatabaseError),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IOError(err)
    }
}

impl From<ShiratsuError> for AppError {
    fn from(err: ShiratsuError) -> Self {
        AppError::ShiratsuError(err)
    }
}

impl From<DatabaseError> for AppError {
    fn from(err: DatabaseError) -> Self {
        AppError::DatabaseError(err)
    }
}

impl std::error::Error for AppError {}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::IOError(err) => write!(f, "IO: {}", err),
            AppError::ShiratsuError(err) => write!(f, "Shiratsu: {}", err),
            AppError::DatabaseError(err) => write!(f, "Database: {}", err),
        }
    }
}

fn get_entries<R: BufRead + Seek>(
    mut reader: R,
) -> Result<Option<(Vec<GameEntry>, &'static str)>, AppError> {
    if let Ok(entries) = GameEntry::try_from_nointro_buf(reader.by_ref()) {
        return Ok(Some((entries, "No-Intro")));
    }
    reader.seek(SeekFrom::Start(0))?;
    if let Ok(entries) = GameEntry::try_from_redump_buf(reader.by_ref()) {
        return Ok(Some((entries, "Redump")));
    };
    reader.seek(SeekFrom::Start(0))?;
    if let Ok(entries) = GameEntry::try_from_tosec_buf(reader.by_ref()) {
        return Ok(Some((entries, "TOSEC")));
    };
    Ok(None)
}

fn setup_logging<T: AsRef<Path>>(log_path: T) -> slog::Logger {
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
    slog::Logger::root(file_drain, o!())
    // slog_scope::set_global_logger(root)
}

lazy_static! {
    static ref SAVE_PB: ProgressBar = ProgressBar::hidden();
    static ref PB_STYLE: ProgressStyle = ProgressStyle::default_spinner()
                                            // .tick_strings(&["⠁ ","⠂ ", "⠄ ", "⡀ ", "⢀ ", "⠠ ", "⠐ ", "⠈ ", ""])
                                            .tick_strings(&["⠋", "⠙", "⠸", "⠴", "⠦", "⠇", "✓"])
                                            .template("{prefix:.bold.dim} {spinner} [{pos}/{len}] {wide_msg}");
    static ref SAVE_PB_STYLE: ProgressStyle = ProgressStyle::default_spinner()
                                            // .tick_strings(&["⠁ ","⠂ ", "⠄ ", "⡀ ", "⢀ ", "⠠ ", "⠐ ", "⠈ ", ""])
                                            .tick_strings(&["⠋", "⠙", "⠸", "⠴", "⠦", "⠇", "✓"])
                                            .template("{prefix:.bold.dim} {spinner} {wide_msg}");
}

fn run_app() -> Result<(), AppError> {
    let args = env::args().skip(1).take(1).next();
    let save_path = args.ok_or(AppError::IOError(io::Error::new(
        ErrorKind::NotFound,
        "No save path was specified.",
    )))?
    ;
    let save_path = Path::new(&save_path);
    
    if save_path.exists() {
        eprintln!(
            "[ERROR] Specified save path {} already exists!",
            save_path.display()
        );
        return Err(AppError::IOError(io::Error::new(
            ErrorKind::AlreadyExists,
            "The specified path already exists.",
        )));
        // return
    }
    let root = setup_logging(format!("{}.log", save_path.display()));
    println!("Generating Shiragame database at {}", save_path.display());
    info!(
        root,
        "Generating Shiragame database at {save_path}",
        save_path = save_path.display()
    );
    let mut db = ShiratsuDatabase::new().unwrap();
    for (platform_id, dir) in ingest::get_paths("dats").into_iter() {
        let reader = BufReader::new(File::open(dir.path())?);
        if let Ok(Some((entries, source))) = get_entries(reader) {
            let pb = ProgressBar::new(entries.len() as u64);
            pb.set_style(PB_STYLE.clone());
            pb.set_message(&format!("{}", dir.path().display()));
            pb.set_draw_delta(entries.len() as u64 / 100);
            info!(
                root,
                "Found {} DAT File at {} ({})",
                source = source,
                path = dir.path().display(),
                platform_id = platform_id.as_ref()
            );
            for game in entries.iter() {
                info!(
                    root,
                    "Adding game entry \"{}\" ({})",
                    entry_name = game.entry_name(),
                    platform_id = platform_id.as_ref(),
                );
                pb.set_message(&format!(
                    "[{}] {}: {}",
                    platform_id.as_ref(),
                    dir.path().display(),
                    game.entry_name()
                ));
                db.add_entry(game, platform_id).unwrap();
                pb.inc(1);
            }
            info!(
                root,
                "Finished processing {}, added {} entries.",
                path = dir.path().display(),
                count = entries.len()
            );
            pb.finish_with_message(&format!(
                "[{}] Finished processing {}, added {} entries.",
                platform_id.as_ref(),
                dir.path().display(),
                entries.len()
            ))
        }
    }

    match db.save(save_path, Some(process_duration)) {
        Ok((uuid, time)) => {
            SAVE_PB.finish_with_message(&format!(
                "Saved Shiragame database {} ({} at {}) ",
                save_path.display(),
                uuid,
                time
            ));
            Ok(())
        }
        Err(err) => {
            eprintln!(
                "[ERROR] Could not save Shiragame database to {}, does it already exist?",
                save_path.display()
            );
            error!(
                root,
                "Could not save Shiragame database to {save_path}, does it already exist?",
                save_path = save_path.display()
            );
            Err(match err {
                DatabaseError::IOError(err) => AppError::IOError(err),
                _ => AppError::DatabaseError(err),
            })
        }
    }
}
fn main() {
    std::process::exit(match run_app() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("[ERROR] Finished with {}", err);
            1
        }
    });
}

pub fn process_duration(p: Progress) {
    if SAVE_PB.is_hidden() {
        SAVE_PB.set_draw_target(ProgressDrawTarget::stderr());
        SAVE_PB.set_style(SAVE_PB_STYLE.clone());
        SAVE_PB.set_length(p.pagecount as u64);
    }
    SAVE_PB.set_position((p.pagecount - p.remaining) as u64);
}
