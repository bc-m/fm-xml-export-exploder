use crate::utils::attributes::get_attributes;
use crate::utils::xml_utils::element_to_string;
use crate::{escape_filename, join_scope_id_and_name, should_skip_line};
use quick_xml::events::BytesStart;
use quick_xml::Reader;
use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

pub(crate) mod attributes;
pub(crate) mod xml_utils;

#[derive(Debug, Default)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub content: String,
}

impl Entity {
    pub fn clear(&mut self) {
        self.id.clear();
        self.name.clear();
        self.content.clear();
    }

    pub fn read_xml_element<R: Read + BufRead>(&mut self, reader: &mut Reader<R>, e: &BytesStart) {
        self.clear();
        for attr in get_attributes(e).unwrap() {
            match attr.0.as_str() {
                "id" => self.id = attr.1.to_string(),
                "name" => {
                    if self.name.is_empty() {
                        self.name = attr.1.to_string()
                    }
                }
                "Display" => self.name = attr.1.to_string(),
                _ => {}
            }
        }
        self.content = element_to_string(reader, e);
    }
}

pub fn write_xml_element_to_file<R: Read + BufRead>(
    reader: &mut Reader<R>,
    e: &BytesStart,
    output_dir: &Path,
    remove_indent_count: usize,
) {
    let mut entity = Entity::default();
    entity.read_xml_element(reader, e);
    write_entity_to_file(output_dir, &entity, remove_indent_count);
}

pub fn write_entity_to_file(output_dir: &Path, entity: &Entity, remove_indent_count: usize) {
    let filename = join_scope_id_and_name(entity.id.as_str(), entity.name.as_str());
    let filename = escape_filename(&filename);

    let output_file_path = output_dir.join(format!("{}.xml", filename));
    write_xml_file(&output_file_path, &entity.content, remove_indent_count);
}

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
