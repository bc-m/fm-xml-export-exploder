use std::{fs, path::PathBuf, time::Instant};

use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;

use crate::config::Flags;
use crate::utils::file_utils::valid_dir_or_throw;
use crate::utils::xml_utils::XmlEventType;
use crate::xml_processor::explode_xml;

mod catalog;
mod config;
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

    /// Enable debug/verbose output. Optionally filter by catalog name.
    /// Example: --verbose "CustomFunctionsCatalog"
    #[arg(short, long, value_name = "FILTER", num_args(0..=1))]
    verbose: Option<Option<String>>,
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
        verbose: args.verbose,
    };

    match &flags.verbose {
        Some(Some(filter)) => println!("🔍 Verbose mode (filtered on {})", filter),
        Some(None) => println!("🔊 Verbose mode (all)"),
        None => (), // silent
    };

    valid_dir_or_throw(&in_dir)?;

    // Read directory contents
    let paths = fs::read_dir(in_dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path())) // Filter out directories and unwrap results
        .filter(|path| path.is_file() && path.extension().unwrap_or_default() == "xml") // Filter XML files
        .collect::<Vec<_>>(); // Collect paths into a vector

    println!("Start processing {} files...", paths.len());

    // Process XML files in parallel
    paths.par_iter().for_each(|path| {
        match explode_xml(path, &out_dir, &flags) {
            Ok(_) => {}
            Err(err) => {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                eprintln!("Failed to process file '{}': {}", file_name, err)
            }
        };
    });

    let duration = start.elapsed();
    if duration.as_secs() > 9 {
        println!("Completed in {:?} seconds.", duration.as_secs());
    } else {
        println!("Completed in {:?} ms.", duration.as_millis());
    }

    Ok(())
}
