use std::{fs, time::Instant};

use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;

use crate::settings::{Args, Settings};
use crate::utils::file_utils::valid_dir_or_throw;
use crate::utils::migrate_old_custom_functions_if_needed;
use crate::utils::xml_utils::XmlEventType;
use crate::xml_processor::explode_xml;

mod catalog;
mod config;
mod custom_function_sanitizer;
mod git;
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

    let settings = Settings::from_args(Args::parse(), &std::env::current_dir()?)?;
    if let Some(config_path) = &settings.config_path {
        println!("Using config {}", config_path.display());
    }
    let in_dir = settings.source;
    let out_dir = settings.target;
    let flags = settings.flags;

    valid_dir_or_throw(&in_dir)?;

    let paths = fs::read_dir(in_dir)?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            (path.is_file() && path.extension().is_some_and(|ext| ext == "xml")).then_some(path)
        })
        .collect::<Vec<_>>();

    migrate_old_custom_functions_if_needed(&out_dir, &flags)?;

    println!("Start processing {} files...", paths.len());

    let db_names: Vec<String> = paths
        .par_iter()
        .filter_map(|path| match explode_xml(path, &out_dir, &flags) {
            Ok(db_name) => db_name,
            Err(err) => {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                eprintln!("Failed to process file '{file_name}': {err}");
                None
            }
        })
        .collect();

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
