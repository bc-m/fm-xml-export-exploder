use std::fs;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::config::Flags;
use crate::script_steps::constants::{id_to_script_step, ScriptStep};
use crate::script_steps::sanitizer::sanitize;
use crate::utils::attributes::get_attribute;
use crate::utils::write_text_file;
use crate::utils::xml_utils::{
    cdata_element_to_string, end_element_to_string, local_name_to_string, start_element_to_string,
    text_element_to_string,
};

#[derive(Debug, Default)]
struct ScriptInfo {
    id: String,
    name: String,
    text: String,
}

#[derive(Debug, Default)]
struct ScriptStepInfo {
    id: u32,
    content: String,
    indent_level_current: usize,
    indent_level_next: usize,
}

/// Process all XML files in the script_steps directory and create sanitized text versions
/// This function mirrors the folder structure of the XML files
pub fn create_sanitized_scripts(
    scripts_xml_out_dir_path: &Path,
    scripts_text_out_dir_path: &Path,
    // out_dir_path: &Path,
    // fm_file_name: &str,
    // saxml_version: &str,
    _script_folder_structure: Option<&crate::utils::FolderStructure>,
    flags: &Flags,
) {
    // Recursively process all XML files in the script_steps directory
    process_directory_recursively(
        scripts_xml_out_dir_path,
        scripts_xml_out_dir_path,
        scripts_text_out_dir_path,
        flags,
    );
}

fn process_directory_recursively(
    current_dir: &Path,
    scripts_xml_out_dir_path: &Path,
    scripts_text_out_dir_path: &Path,
    flags: &Flags,
) {
    if let Ok(entries) = fs::read_dir(current_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("xml") {
                process_script_xml_file(
                    &path,
                    scripts_xml_out_dir_path,
                    scripts_text_out_dir_path,
                    flags,
                );
            } else if path.is_dir() {
                // Recursively process subdirectories
                process_directory_recursively(
                    &path,
                    scripts_xml_out_dir_path,
                    scripts_text_out_dir_path,
                    flags,
                );
            }
        }
    }
}

fn process_script_xml_file(
    xml_file_path: &Path,
    scripts_xml_out_dir_path: &Path,
    scripts_text_out_dir_path: &Path,
    flags: &Flags,
) {
    // Read the XML file content
    let xml_content = match fs::read_to_string(xml_file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", xml_file_path.display(), e);
            return;
        }
    };

    // Parse the script and create sanitized text
    let script_info = parse_script_xml(&xml_content, flags);
    if let Some(script_info) = script_info {
        // Determine the relative path from the XML file to maintain folder structure
        let relative_path = xml_file_path
            .strip_prefix(scripts_xml_out_dir_path)
            .unwrap_or(xml_file_path);
        let output_file_path = scripts_text_out_dir_path.join(relative_path);

        // Ensure the output directory exists
        if let Some(parent) = output_file_path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|err| {
                panic!("Error creating directory {}: {}", parent.display(), err)
            });
        }

        // Change extension to .txt
        let output_file_path = output_file_path.with_extension("txt");
        write_text_file(&output_file_path, &script_info.text);
    }
}

fn parse_script_xml(xml_content: &str, flags: &Flags) -> Option<ScriptInfo> {
    let mut script_info = ScriptInfo::default();
    let mut in_step = false;
    let mut step_info = ScriptStepInfo::default();

    let mut reader = Reader::from_str(xml_content);
    let mut buf = Vec::new();
    let mut depth = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                println!("Error parsing XML: {e}");
                break;
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                depth += 1;

                if depth == 1 && e.name().as_ref() == b"Script" {
                    // Script element - we'll get ID and name from ScriptReference
                } else if depth == 2 && e.name().as_ref() == b"ScriptReference" {
                    // Extract script ID and name from ScriptReference
                    for attr in crate::utils::attributes::get_attributes(&e).unwrap() {
                        match attr.0.as_str() {
                            "id" => script_info.id = attr.1.to_string(),
                            "name" => {
                                script_info.name = quick_xml::escape::unescape(attr.1.as_str())
                                    .unwrap()
                                    .to_string()
                            }
                            _ => {}
                        }
                    }
                } else if depth == 3 && local_name_to_string(e.name().as_ref()) == "Step" {
                    in_step = true;
                    step_info.indent_level_current = step_info.indent_level_next;
                    step_info.id = get_attribute(&e, "id").unwrap().parse::<u32>().unwrap();

                    if get_attribute(&e, "enable").unwrap_or("True".to_string()) == "True" {
                        match id_to_script_step(&step_info.id) {
                            ScriptStep::IfStart | ScriptStep::LoopStart => {
                                step_info.indent_level_next += 1
                            }
                            ScriptStep::IfElse | ScriptStep::Else => {
                                step_info.indent_level_current -= 1
                            }
                            ScriptStep::IfEnd | ScriptStep::LoopEnd => {
                                step_info.indent_level_current -= 1;
                                step_info.indent_level_next = step_info.indent_level_current;
                            }
                            _ => {}
                        }
                    }
                }

                if in_step {
                    step_info
                        .content
                        .push_str(start_element_to_string(&e, flags).as_str());
                }
            }
            Ok(Event::End(e)) => {
                depth -= 1;

                if depth == 0 {
                    break;
                }

                if in_step {
                    step_info
                        .content
                        .push_str(end_element_to_string(&e).as_str());
                }

                if depth == 2 && local_name_to_string(e.name().as_ref()) == "Step" {
                    let is_comment = id_to_script_step(&step_info.id) == ScriptStep::Comment;
                    match sanitize(&step_info.id, &step_info.content) {
                        None => {}
                        Some(text) => {
                            let mut first_line_done = false;
                            let mut add_indent = 0;
                            for line in text.split('\r') {
                                let mut indent =
                                    "\t".repeat(step_info.indent_level_current + add_indent);
                                if is_comment && first_line_done {
                                    indent.push_str(&" ".repeat(2));
                                }

                                script_info.text.push_str(&format!("{indent}{line}\n"));
                                if !first_line_done {
                                    first_line_done = true;
                                    if !is_comment {
                                        add_indent = 4
                                    };
                                }
                            }
                        }
                    }
                    step_info.indent_level_current = step_info.indent_level_next;
                    step_info.content.clear()
                }
            }
            Ok(Event::CData(e)) => {
                if in_step {
                    step_info
                        .content
                        .push_str(cdata_element_to_string(&e).as_str());
                }
            }
            Ok(Event::Comment(e)) | Ok(Event::Text(e)) => {
                if in_step {
                    step_info
                        .content
                        .push_str(text_element_to_string(&e, true).as_str());
                }
            }
            _ => {}
        }

        buf.clear()
    }

    if script_info.id.is_empty() {
        None
    } else {
        Some(script_info)
    }
}
