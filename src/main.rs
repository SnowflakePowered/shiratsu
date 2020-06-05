mod database;
mod ingest;
mod sortrules;

use shiratsu_lib::{
    parse::*,
    parse::{nointro::*, redump::*, tosec::*},
    stone::StonePlatforms,
};

use anyhow::Result;

use slog::{error, info, o, Drain};

use std::borrow::Cow;
use std::env;
use std::fs::{create_dir, File, OpenOptions};
use std::io::{self, BufRead, BufReader, ErrorKind, Seek, SeekFrom};
use std::path::Path;
use std::time::Instant;

use database::{DatabaseError, ShiratsuDatabase};

use colored::*;
use console::style;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use rusqlite::backup::Progress;

use lazy_static::lazy_static;
use lazy_static_include::{
    lazy_static_include_str, lazy_static_include_str_impl, lazy_static_include_str_inner,
};

use glob::glob_with;
use glob::MatchOptions;

fn get_entries<R: BufRead + Seek>(mut reader: R) -> Result<Option<(Vec<GameEntry>, &'static str)>> {
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
                                            .tick_strings(&["⠋", "⠙", "⠸", "⠴", "⠦", "⠇", &format!("{}", "✓".green())])
                                            .template("{prefix:.bold.dim} {spinner} [{pos}/{len}] {wide_msg}");
    static ref SAVE_PB_STYLE: ProgressStyle = ProgressStyle::default_spinner()
                                            // .tick_strings(&["⠁ ","⠂ ", "⠄ ", "⡀ ", "⢀ ", "⠠ ", "⠐ ", "⠈ ", ""])
                                            .tick_strings(&["⠋", "⠙", "⠸", "⠴", "⠦", "⠇", &format!("{}", "✓".green())])
                                            .template("{prefix:.bold.dim} {spinner} {wide_msg}");
}

lazy_static_include_str!(SORTING_RULES, "sortrules.yml");

fn create_folders() -> Result<()> {
    let mut current_dir = env::current_dir()?;
    current_dir.push("dats");
    println!("Creating folder structure in {}", "dats".cyan());
    if !current_dir.exists() {
        println!(
            " {} Created directory {}",
            "✓".green(),
            style(current_dir.display()).cyan()
        );
        create_dir(&current_dir)?;
    }
    for platform_id in StonePlatforms::get().ids() {
        current_dir.push(platform_id.as_ref());
        if !current_dir.exists() {
            create_dir(&current_dir)?;
            println!(
                " {} Created directory {}",
                "✓".green(),
                style(current_dir.display()).cyan()
            );
        } else {
            println!(
                " {} Directory {} already exists",
                "✓".green(),
                style(current_dir.display()).cyan()
            );
        }
        current_dir.pop();
    }
    current_dir.pop();
    current_dir.push("unsorted");
    if !current_dir.exists() {
        create_dir(&current_dir)?;
        println!(
            " {} Created unsorted directory {}",
            "✓".green(),
            style(current_dir.display()).cyan()
        );
    } else {
        println!(
            " {} Unsorted directory {} already exists",
            "✓".green(),
            style(current_dir.display()).cyan()
        );
    }
    println!(
        " {} -- Created required folder structure",
        "✓ Success".green()
    );
    Ok(())
}

fn create_db<S: AsRef<str>>(save_path: S) -> Result<()> {
    let now = Instant::now();
    let save_path = Path::new(save_path.as_ref());

    if save_path.exists() {
        eprintln!(
            "Specified save path {} already exists!",
            style(save_path.display()).cyan()
        );
        return Err(anyhow::Error::new(io::Error::new(
            ErrorKind::AlreadyExists,
            "The specified path already exists.",
        )));
    }
    let root = setup_logging(format!("{}.log", save_path.display()));
    println!(
        "Generating Shiragame database at {}",
        style(save_path.display()).cyan(),
    );
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
                style(dir.path().display()).cyan(),
                entries.len()
            ))
        }
    }

    match db.save(save_path, Some(process_duration)) {
        Ok((uuid, time)) => {
            SAVE_PB.finish_with_message(&format!(
                "{} -- Saved Shiragame database {} ({} at {}) in {} seconds",
                style("Success").green(),
                style(save_path.display()).cyan(),
                style(uuid).green().bold(),
                style(time).green(),
                style(now.elapsed().as_secs()).cyan(),
            ));
            Ok(())
        }
        Err(err) => {
            eprintln!(
                "Could not save Shiragame database to {}, does it already exist?",
                style(save_path.display()).cyan()
            );
            error!(
                root,
                "Could not save Shiragame database to {save_path}, does it already exist?",
                save_path = save_path.display()
            );
            Err(match err {
                DatabaseError::IOError(err) => anyhow::Error::new(err),
                DatabaseError::SqliteError(err) => anyhow::Error::new(err),
            })
        }
    }
}

fn sort_dats() -> Result<()> {
    let now = Instant::now();
    let rules = std::fs::read_to_string("sortrules.yml")
        .map_or_else(|_| Cow::Borrowed(*SORTING_RULES), |s| Cow::Owned(s));
    let sort_rules_src;

    match &rules {
        Cow::Borrowed(_) => {
            sort_rules_src = "internal sorting rules";
            println!("Loading {}", style(sort_rules_src).cyan())
        }
        Cow::Owned(_) => {
            sort_rules_src = "sortrules.yml";
            println!(
                "Loading sorting rules from {}",
                style("sortrules.yml").cyan()
            )
        }
    }

    let rules = sortrules::load_map(rules)?;

    println!(
        " {} Loaded sorting rules from {}",
        "✓".green(),
        sort_rules_src.cyan(),
    );

    create_folders()?;
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
                println!(
                    " {} Sorted {:#?} as {}",
                    "✓".green(),
                    style(filename).cyan(),
                    platform_id.as_ref()
                );
                count += 1;
                current_dir.pop();
            }
        }
        current_dir.pop();
    }
    println!(
        " {} -- Sorted {} DATs in {} seconds",
        "✓ Success".green(),
        style(count).cyan(),
        style(now.elapsed().as_secs()).cyan(),
    );
    Ok(())
}

fn run_app() -> Result<()> {
    let args = env::args().skip(1).take(1).next();
    let command = args.ok_or(io::Error::new(
        ErrorKind::NotFound,
        "No save path was specified.",
    ))?;

    match command.as_str() {
        "sort" => sort_dats(),
        save_path => create_db(save_path),
    }
}

fn main() {
    std::process::exit(match run_app() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("{} -- {}", style(" ✘ Error").red(), err);
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
