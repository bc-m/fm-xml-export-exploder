use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::base_table_catalog::parse_base_table_catalog;
use crate::custom_function_catalog::xml_explode_custom_function_catalog;
use crate::external_data_source_catalog::xml_extract_external_data_sources;
use crate::layout_catalog::xml_explode_layout_catalog;
use crate::relationship_catalog::xml_explode_relationship_catalog;
use crate::script_catalog::parse_script_directories;
use crate::script_steps_catalog::xml_explode_script_catalog;
use crate::table_catalog::xml_explode_table_catalog;
use crate::theme_catalog::xml_explode_theme_catalog;
use crate::utils::attributes::get_attribute;
use crate::value_list_catalog::xml_explode_value_list_catalog;
use encoding_rs_io::DecodeReaderBytes;
use rayon::prelude::*;
use std::collections::HashMap;
use std::{
    env,
    error::Error,
    fs,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    time::Instant,
};

mod base_table_catalog;
mod calculations;
mod custom_function_catalog;
mod external_data_source_catalog;
mod layout_catalog;
mod relationship_catalog;
mod script_catalog;
mod script_steps;
mod script_steps_catalog;
mod table_catalog;
mod theme_catalog;
mod utils;
mod value_list_catalog;

fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    let in_dir = env::args().nth(1).ok_or("Input directory not provided")?;
    let out_dir = env::args().nth(2).ok_or("Output directory not provided")?;

    let in_dir = Path::new(&in_dir);
    if !is_valid_directory(in_dir) {
        println!("'{}' is not a valid directory.", in_dir.display());
        return Ok(());
    }

    let out_dir = Path::new(&out_dir);
    if !is_valid_directory(out_dir) {
        println!("'{}' is not a valid directory.", out_dir.display());
        return Ok(());
    }

    // Read directory contents
    let paths = fs::read_dir(in_dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path())) // Filter out directories and unwrap results
        .filter(|path| path.is_file() && path.extension().unwrap_or_default() == "xml") // Filter XML files
        .collect::<Vec<_>>(); // Collect paths into a vector

    println!("Start processing {} files...", paths.len());

    // Process XML files in parallel
    paths.par_iter().for_each(|path| explode_xml(path, out_dir));

    let duration = start.elapsed();
    if duration.as_secs() > 9 {
        println!("Completed in {:?} seconds.", duration.as_secs());
    } else {
        println!("Completed in {:?} ms.", duration.as_millis());
    }

    Ok(())
}

fn is_valid_directory(dir_path: &Path) -> bool {
    fs::metadata(dir_path)
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false)
}

fn explode_xml(fm_export_file_path: &PathBuf, out_dir_path: &Path) {
    let start = Instant::now();
    let fm_export_file_name = fm_export_file_path.file_name().unwrap().to_str().unwrap();

    // Open XML file
    let file = match File::open(fm_export_file_path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!(
                "Error opening file {}: {}",
                fm_export_file_path.display(),
                err
            );
            return;
        }
    };

    // Initialize variables
    let mut fm_file_name = String::new();
    let mut depth = 0;
    let mut script_id_path_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut table_name_id_map: HashMap<String, String> = HashMap::new();

    // Iterate over XML events
    let decode_reader = BufReader::new(DecodeReaderBytes::new(file));
    let mut reader = Reader::from_reader(decode_reader);
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                println!("Error in {}: {}", fm_export_file_path.display(), e);
                break;
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                match depth {
                    0 => match e.name().as_ref() {
                        b"FMDynamicTemplate" | b"FMSaveAsXML" => {
                            fm_file_name = get_attribute(&e, "File")
                                .unwrap()
                                .strip_suffix(".fmp12")
                                .unwrap()
                                .to_string()
                        }
                        _ => {
                            eprintln!("Unsupported XML-format in file {}", fm_export_file_name);
                            return;
                        }
                    },
                    2 => {
                        if e.name().as_ref() == b"ModifyAction" {
                            break;
                        }
                    }
                    3 => match e.name().as_ref() {
                        b"BaseTableCatalog" => {
                            table_name_id_map = parse_base_table_catalog(&mut reader, &e);
                            continue;
                        }
                        b"LayoutCatalog" => {
                            xml_explode_layout_catalog(
                                &mut reader,
                                &e,
                                out_dir_path,
                                &fm_file_name,
                            );
                            continue;
                        }
                        b"FieldsForTables" => {
                            xml_explode_table_catalog(
                                &mut reader,
                                &e,
                                out_dir_path,
                                &fm_file_name,
                                &table_name_id_map,
                            );
                            continue;
                        }
                        b"CalcsForCustomFunctions" => {
                            xml_explode_custom_function_catalog(
                                &mut reader,
                                &e,
                                out_dir_path,
                                &fm_file_name,
                            );
                            continue;
                        }
                        b"StepsForScripts" => {
                            xml_explode_script_catalog(
                                &mut reader,
                                &e,
                                out_dir_path,
                                &fm_file_name,
                                &script_id_path_map,
                            );
                            continue;
                        }
                        b"ScriptCatalog" => {
                            script_id_path_map = parse_script_directories(&mut reader, &e);
                            continue;
                        }
                        b"ExternalDataSourceCatalog" => {
                            xml_extract_external_data_sources(
                                &mut reader,
                                &e,
                                out_dir_path,
                                &fm_file_name,
                            );
                            continue;
                        }
                        b"ValueListCatalog" => {
                            xml_explode_value_list_catalog(
                                &mut reader,
                                &e,
                                out_dir_path,
                                &fm_file_name,
                            );
                            continue;
                        }
                        b"RelationshipCatalog" => {
                            xml_explode_relationship_catalog(
                                &mut reader,
                                &e,
                                out_dir_path,
                                &fm_file_name,
                            );
                            continue;
                        }
                        b"ThemeCatalog" => {
                            xml_explode_theme_catalog(&mut reader, &e, out_dir_path, &fm_file_name);
                            continue;
                        }
                        _ => {}
                    },
                    _ => {}
                }
                depth += 1;
            }
            Ok(Event::End(_)) => {
                depth -= 1;
            }
            _ => {}
        }
        buf.clear()
    }

    println!(
        "{} finished in {} ms.",
        fm_export_file_name,
        start.elapsed().as_millis()
    );
}

fn join_scope_id_and_name(scope_id: &str, scope_name: &str) -> String {
    format!("{} - ID {}", scope_name, scope_id)
}

fn escape_filename(filename: &str) -> String {
    // Replace special characters with underscores
    let escaped_filename = filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '{' => '_',
            _ => c,
        })
        .collect::<String>();

    escaped_filename.trim().to_string()
}

fn should_skip_line(line: &str) -> bool {
    // de-noise TODO: make optional
    if line.contains("<TagList></TagList>") {
        return true;
    };
    if line.contains("<OwnerID></OwnerID>") {
        return true;
    };
    if line.contains("<Options>0</Options>") {
        return true;
    }; // Default
    if line.contains("<Options>1048576</Options>") {
        return true;
    }; // Breakpoint
    if line.contains("<UUID>") && line.contains("</UUID>") {
        return true;
    };

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use fs::remove_dir_all;
    use insta::assert_snapshot;
    use similar_asserts::assert_eq;
    use std::io::Read;
    use walkdir::WalkDir;

    #[test]
    fn test_escape_filename() {
        assert_eq!(escape_filename("filename.xml"), "filename.xml");
        assert_eq!(escape_filename("file/name.xml"), "file_name.xml");
        assert_eq!(escape_filename("file|name.xml"), "file_name.xml");
    }

    #[test]
    fn snapshot_test() {
        let snapshot_dir = Path::new("./tests/snapshots");
        let input_dir = Path::new("./tests/xml");
        let output_dir = Path::new("./out");
        let _ = remove_dir_all(output_dir);

        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(snapshot_dir);
        settings.set_prepend_module_to_snapshot(false);

        let paths = fs::read_dir(input_dir)
            .unwrap()
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| path.is_file())
            .filter(|e| e.file_name().unwrap() != ".DS_Store")
            .collect::<Vec<_>>();

        for path in paths {
            explode_xml(&path, output_dir);
        }

        let output_files: Vec<PathBuf> = WalkDir::new(output_dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| e.file_name().to_str().unwrap() != ".DS_Store")
            .map(|e| e.path().to_path_buf())
            .collect();

        let mut output_file_paths = output_files
            .iter()
            .map(|file| file.strip_prefix(output_dir).unwrap())
            .map(|file| file.to_string_lossy())
            .collect::<Vec<_>>();
        output_file_paths.sort();

        let snapshot_files: Vec<PathBuf> = WalkDir::new(snapshot_dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| e.file_name().to_str().unwrap().ends_with(".snap"))
            .map(|e| e.path().to_path_buf())
            .collect();

        let mut snapshot_file_paths = snapshot_files
            .iter()
            .map(|file| file.strip_prefix(snapshot_dir).unwrap())
            .map(|file| {
                file.to_string_lossy()
                    .strip_suffix(".snap")
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<_>>();
        snapshot_file_paths.sort();

        assert_eq!(output_file_paths.join("\n"), snapshot_file_paths.join("\n"));

        for output_file in &output_files {
            let output_content = String::from_utf8(read_file(output_file)).unwrap();
            let output_file_slug = output_file.strip_prefix(output_dir).unwrap();

            settings.set_snapshot_path(
                Path::new("../tests/snapshots")
                    .join(output_file_slug)
                    .parent()
                    .unwrap(),
            );

            settings.bind(|| {
                assert_snapshot!(output_file.file_name().unwrap().to_str(), output_content);
            });
        }
    }

    fn read_file(file_path: &PathBuf) -> Vec<u8> {
        let mut file = fs::File::open(file_path).expect("Failed to open file");
        let mut content = Vec::new();
        file.read_to_end(&mut content).expect("Failed to read file");
        content
    }
}
