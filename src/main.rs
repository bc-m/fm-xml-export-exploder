use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;

use crate::input::input_files;
use crate::settings::{Args, Command, Settings};
use crate::utils::file_utils::valid_dir_or_throw;
use crate::utils::migrate_old_custom_functions_if_needed;
use crate::utils::output_files::OutputFiles;
use crate::utils::xml_utils::XmlEventType;
use crate::xml_processor::{explode_file, remove_stale_output};

mod catalog;
mod config;
mod custom_function_sanitizer;
mod git;
mod input;
mod reassemble;
mod script_sanitizer;
mod script_steps;
mod settings;
mod supporting;
mod tests;
mod utils;
mod xml_processor;

#[derive(Debug, Default)]
pub struct Skeleton {
    pub content: String,
    pub previous_line: String,
    pub previous_event_type: XmlEventType,
}

fn main() -> Result<()> {
    let start = Instant::now();

    let args = Args::parse();
    if let Some(Command::Reassemble { source, target }) = &args.command {
        return reassemble::reassemble(source, target);
    }
    let settings = Settings::from_args(args, &std::env::current_dir()?)?;
    if let Some(config_path) = &settings.config_path {
        println!("Using config {}", config_path.display());
    }
    let in_dir = settings.source;
    let out_dir = settings.target;
    let flags = settings.flags;

    valid_dir_or_throw(&in_dir)?;

    let paths = input_files(&in_dir)?;

    migrate_old_custom_functions_if_needed(&out_dir, &flags)?;

    println!("Start processing {} files...", paths.len());

    // Shared by all files: a database split into one file per catalog is written by
    // several files
    let output = OutputFiles::default();
    let any_failed = AtomicBool::new(false);
    let mut db_names: Vec<String> = paths
        .par_iter()
        .filter_map(|path| match explode_file(path, &out_dir, &flags, &output) {
            Ok(db_name) => db_name,
            Err(err) => {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                eprintln!("Failed to process file '{file_name}': {err}");
                any_failed.store(true, Ordering::Relaxed);
                None
            }
        })
        .collect();
    db_names.sort();
    db_names.dedup();

    // The output is updated in place: remove what the exports no longer contain. A failed
    // file may belong to any database, so nothing is removed then.
    if any_failed.load(Ordering::Relaxed) {
        eprintln!("Some files failed, so outdated output files were not removed.");
    } else {
        for db_name in &db_names {
            remove_stale_output(&out_dir, &flags, db_name, &output)?;
        }
    }

    // Commit only after all files are written: in domain mode, several databases share
    // the same repository
    if settings.git {
        git::commit_output(&git::repositories(&out_dir, &flags.output_tree, &db_names));
    }

    let duration = start.elapsed();
    match duration.as_secs() {
        10.. => println!("Completed in {} seconds.", duration.as_secs()),
        _ => println!("Completed in {} ms.", duration.as_millis()),
    }

    Ok(())
}
