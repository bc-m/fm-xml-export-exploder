use std::{fs, path::PathBuf, time::Instant};

use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;

use crate::config::{Flags, OutputTree};
use crate::utils::file_utils::valid_dir_or_throw;
use crate::utils::migrate_old_custom_functions_if_needed;
use crate::utils::xml_utils::XmlEventType;
use crate::xml_processor::explode_xml;

mod catalog;
mod config;
mod custom_function_sanitizer;
mod script_sanitizer;
mod script_steps;
mod supporting;
mod tests;
mod utils;
mod xml_processor;

/// Parse all as XML exported FileMaker solutions from source directory and explode them to target directory.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// The source directory to read input
    source: PathBuf,

    /// The target directory to write output
    target: PathBuf,

    /// Parse all lines (or skip less important ones to reduce noise)
    #[arg(short, long)]
    all_lines: bool,

    /// Retain all information from the main xml (or skip less important catalogs and attributes)
    #[arg(short, long)]
    lossless: bool,

    /// Specify the output tree root folder: domain or db (default)
    #[arg(short = 't', long = "output_tree", value_enum, default_value_t = OutputTree::Db)]
    output_tree: OutputTree,
}

#[derive(Debug, Default)]
pub struct Skeleton {
    pub content: String,
    pub previous_line: String,
    pub previous_event_type: XmlEventType,
}

fn main() -> Result<()> {
    let start = Instant::now();

    let args = Args::parse();
    let in_dir = args.source;
    let out_dir = args.target;
    let flags = Flags {
        parse_all_lines: args.all_lines,
        lossless: args.lossless,
        output_tree: args.output_tree,
    };

    valid_dir_or_throw(&in_dir)?;

    let paths = fs::read_dir(in_dir)?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            (path.is_file() && path.extension().is_some_and(|ext| ext == "xml")).then_some(path)
        })
        .collect::<Vec<_>>();

    migrate_old_custom_functions_if_needed(&out_dir, &flags)?;

    println!("Start processing {} files...", paths.len());

    paths.par_iter().for_each(|path| {
        if let Err(err) = explode_xml(path, &out_dir, &flags) {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            eprintln!("Failed to process file '{file_name}': {err}");
        }
    });

    let duration = start.elapsed();
    match duration.as_secs() {
        10.. => println!("Completed in {} seconds.", duration.as_secs()),
        _ => println!("Completed in {} ms.", duration.as_millis()),
    }

    Ok(())
}
