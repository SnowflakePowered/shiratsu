use crate::Event;
use colored::*;
use console::style;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use lazy_static::lazy_static;
use rusqlite::backup::Progress;
use shiratsu_naming::naming::NameError;
use slog::{error, info, warn};
use shiratsu_dat::DatError;

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

pub fn process_duration(p: Progress) {
    if SAVE_PB.is_hidden() {
        SAVE_PB.set_draw_target(ProgressDrawTarget::stderr());
        SAVE_PB.set_style(SAVE_PB_STYLE.clone());
        SAVE_PB.set_length(p.pagecount as u64);
        SAVE_PB.set_message("Saving...")
    }
    SAVE_PB.set_position((p.pagecount - p.remaining) as u64);
}

pub fn print_event(e: Event) {
    match e {
        Event::CreatingFolderStructure => {
            println!("Creating folder structure in {}", "dats".cyan())
        }
        Event::CreatedDirectory(p) => println!(
            " {} Created directory {}",
            "✓".green(),
            style(p.display()).cyan()
        ),
        Event::DirectoryAlreadyExists(p) => println!(
            " {} Directory {} already exists",
            "✓".green(),
            style(p.display()).cyan()
        ),
        Event::CreatedUnsortedDirectory(p) => println!(
            " {} Created unsorted directory {}",
            "✓".green(),
            style(p.display()).cyan()
        ),
        Event::UnsortedDirectoryAlreadyExists(p) => println!(
            " {} Unsorted directory {} already exists",
            "✓".green(),
            style(p.display()).cyan()
        ),
        Event::CreateFoldersSuccess => println!(
            " {} -- Created required folder structure",
            "✓ Success".green()
        ),
        Event::DatabaseSavePathAlreadyExists(p) => eprintln!(
            "Specified save path {} already exists!",
            style(p.display()).cyan()
        ),
        Event::GeneratingDatabase(p, root) => {
            info!(
                root,
                "Generating Shiragame database at {save_path}",
                save_path = p.display()
            );
            println!(
                "Generating Shiragame database at {}",
                style(p.display()).cyan(),
            )
        }
        Event::FoundDatFile(pb, p, len, platform_id, source, root, filelog) => {
            info!(
                root,
                "Found {} DAT File at {} ({})",
                source = source,
                path = p.display(),
                platform_id = platform_id.as_ref()
            );

            info!(
                filelog,
                "{}: {}",
                platform_id = platform_id.as_ref(),
                path = p.display(),
            );

            pb.set_style(PB_STYLE.clone());
            pb.set_message(&format!("{}", p.display()));
            pb.set_draw_delta(len / 100);
        }
        Event::ProcessEntry(pb, platform_id, p, entry_name, root) => {
            info!(
                root,
                "Adding game entry \"{}\" ({})",
                entry_name = entry_name,
                platform_id = platform_id.as_ref(),
            );
            pb.set_message(&format!(
                "[{}] {}: {}",
                platform_id.as_ref(),
                p.display(),
                entry_name
            ));
        }
        Event::ProcessEntrySuccess(pb) => pb.inc(1),
        Event::DatProcessingSuccess(pb, platform_id, p, len, root) => {
            info!(
                root,
                "Finished processing {}, added {} entries.",
                path = p.display(),
                count = len
            );

            pb.finish_with_message(&format!(
                "[{}] Finished processing {}, added {} entries.",
                platform_id.as_ref(),
                style(p.display()).cyan(),
                len
            ))
        }
        Event::DbSaveSuccess(p, uuid, time, now) => {
            SAVE_PB.finish_with_message(&format!(
                "{} -- Saved Shiragame database {} ({} at {}) in {} seconds",
                style("Success").green(),
                style(p.display()).cyan(),
                style(uuid).green().bold(),
                style(time).green(),
                style(now).cyan(),
            ));
        }
        Event::DbSaveError(p, root) => {
            error!(
                root,
                "Could not save Shiragame database to {save_path}, does it already exist?",
                save_path = p.display()
            );

            eprintln!(
                "Could not save Shiragame database to {}, does it already exist?",
                style(p.display()).cyan()
            );
        }
        Event::LoadExternalSortingRules => {
            println!("Loading {}", style("internal sorting rules").cyan())
        }
        Event::LoadInternalSortingRules => println!(
            "Loading sorting rules from {}",
            style("sortrules.yml").cyan()
        ),
        Event::LoadedSortingRules(s) => {
            println!(" {} Loaded sorting rules from {}", "✓".green(), s.cyan(),)
        }
        Event::SortedFile(f, platform_id) => {
            println!(
                " {} Sorted {:#?} as {}",
                "✓".green(),
                style(f).cyan(),
                platform_id.as_ref()
            );
        }
        Event::SortingSuccess(count, now) => {
            println!(
                " {} -- Sorted {} DATs in {} seconds",
                "✓ Success".green(),
                style(count).cyan(),
                style(now).cyan(),
            );
        }
        Event::NoEntriesFound(filename, root) => {
            warn!(root, "No entries found for DAT {:#?}", filename);
            eprintln!(
                " {} -- No entries found for DAT {:#?}",
                "! Warning".yellow(),
                style(filename).cyan(),
            );
        }
        Event::ParseEntryError(err, root) => match err {
            DatError::NameError(NameError::ParseError(convention, filename)) => {
                warn!(
                    root,
                    "Could not parse {} under the {:?} naming convention", filename, convention
                );
                eprintln!(
                    " {} -- Could not parse {} under the {:?} naming convention",
                    "! Warning".yellow(),
                    style(filename).cyan(),
                    style(convention).cyan()
                );
            }
            _ => {
                warn!(root, "Entry failed to parse: {:?}", err);
                eprintln!(
                    " {} -- Entry failed to parse: {:?}",
                    "! Warning".yellow(),
                    style(err).cyan(),
                )
            }
        },
    }
}
