use crate::should_skip_line;
use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub(crate) mod attributes;
pub(crate) mod xml_utils;

pub fn initialize_out_dir(out_dir_path: &Path) {
    if out_dir_path.exists() {
        fs::remove_dir_all(out_dir_path).unwrap_or_else(|err| {
            panic!(
                "Error creating directory {}: {}",
                out_dir_path.display(),
                err
            )
        });
    }
    fs::create_dir_all(out_dir_path).unwrap_or_else(|err| {
        panic!(
            "Error creating directory {}: {}",
            out_dir_path.display(),
            err
        )
    });
}

pub fn create_dir(dir_path: &Path) {
    fs::create_dir_all(dir_path)
        .unwrap_or_else(|err| panic!("Error creating directory {}: {}", dir_path.display(), err));
}

pub fn write_xml_file(output_file_path: &Path, content: &str, remove_indent_count: usize) {
    let mut file_content = String::new();
    let reader = BufReader::new(content.as_bytes());
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        if should_skip_line(&line) {
            continue;
        }
        file_content.push_str(
            line.strip_prefix(&"\t".repeat(remove_indent_count))
                .unwrap_or(&line),
        );
        file_content.push('\n');
    }

    write_file(output_file_path, &file_content);
}

pub fn write_text_file(output_file_path: &Path, content: &str) {
    let mut file_content = String::new();
    let regex = Regex::new(r"\r\n|\n\r|\r|\n").unwrap();
    for line in regex.split(content) {
        file_content.push_str(line);
        file_content.push('\n');
    }

    write_file(output_file_path, &file_content);
}

fn write_file(output_file_path: &Path, file_content: &String) {
    match File::create(output_file_path) {
        Ok(ref mut output_file) => {
            write!(output_file, "{}", file_content).expect("Failed to write to file");
            output_file.flush().expect("Failed to flush output file");
        }
        Err(err) => {
            eprintln!(
                "Error creating file {}: {}",
                output_file_path.display(),
                err
            );
        }
    }
}
