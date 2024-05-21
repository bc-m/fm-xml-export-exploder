use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, Read};
use std::path::Path;

use quick_xml::escape::unescape;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;

use crate::script_steps::constants::{id_to_script_step, ScriptStep};
use crate::script_steps::sanitizer::sanitize;
use crate::utils::attributes::{get_attribute, get_attributes};
use crate::utils::xml_utils::{
    cdata_element_to_string, end_element_to_string, local_name_to_string, start_element_to_string,
    text_element_to_string,
};
use crate::utils::{initialize_out_dir, write_text_file, write_xml_file};
use crate::{escape_filename, join_scope_id_and_name};

#[derive(Debug, Default)]
struct ScriptInfo {
    id: String,
    name: String,
    xml: String,
    text: String,
    path: Vec<String>,
}

#[derive(Debug, Default)]
struct ScriptStepInfo {
    id: u32,
    content: String,
    indent_level_current: usize,
    indent_level_next: usize,
}

pub fn xml_explode_script_catalog<R: Read + BufRead>(
    reader: &mut Reader<R>,
    _: &BytesStart,
    out_dir_path: &Path,
    fm_file_name: &str,
    script_id_path_map: &HashMap<String, Vec<String>>,
) {
    let scripts_xml_out_dir_path = out_dir_path.join("scripts").join(fm_file_name);
    let scripts_text_out_dir_path = out_dir_path.join("scripts_sanitized").join(fm_file_name);
    initialize_out_dir(&scripts_xml_out_dir_path);
    initialize_out_dir(&scripts_text_out_dir_path);

    let mut script_info = ScriptInfo::default();

    let mut in_step = false;
    let mut step_info = ScriptStepInfo::default();

    let mut depth = 1;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                println!("Error {}", e);
                break;
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                depth += 1;

                if depth == 2 {
                    script_info.id.clear();
                    script_info.name.clear();
                    script_info.xml.clear();
                    script_info.text.clear();
                    step_info.content.clear();
                } else if depth == 3 && e.name().as_ref() == b"ScriptReference" {
                    for attr in get_attributes(&e).unwrap() {
                        match attr.0.as_str() {
                            "id" => script_info.id = attr.1.to_string(),
                            "name" => {
                                script_info.name = unescape(attr.1.as_str()).unwrap().to_string()
                            }
                            _ => {}
                        }
                    }

                    match script_id_path_map.get(&script_info.id) {
                        None => {}
                        Some(path) => {
                            script_info.path.clone_from(path);
                        }
                    };
                } else if depth == 4 && local_name_to_string(e.name().as_ref()) == "Step" {
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

                script_info
                    .xml
                    .push_str(start_element_to_string(&e).as_str());

                if in_step {
                    step_info
                        .content
                        .push_str(start_element_to_string(&e).as_str());
                }
            }
            Ok(Event::End(e)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }

                script_info.xml.push_str(end_element_to_string(&e).as_str());

                if depth == 1 && e.name().as_ref() == b"Script" {
                    write_script_to_file(out_dir_path, fm_file_name, &script_info);
                    script_info.id.clear();
                    script_info.name.clear();
                    script_info.text.clear();
                    script_info.xml.clear();
                    script_info.path.clear();
                    continue;
                }

                if in_step {
                    step_info
                        .content
                        .push_str(end_element_to_string(&e).as_str());
                }

                if depth == 3 && local_name_to_string(e.name().as_ref()) == "Step" {
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

                                script_info.text.push_str(&format!("{}{}\n", indent, line));
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
                script_info
                    .xml
                    .push_str(cdata_element_to_string(&e).as_str());

                if in_step {
                    step_info
                        .content
                        .push_str(cdata_element_to_string(&e).as_str());
                }
            }
            Ok(Event::Comment(e)) | Ok(Event::Text(e)) => {
                script_info
                    .xml
                    .push_str(text_element_to_string(&e, true).as_str());

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
}

fn write_script_to_file(dir_path: &Path, fm_file_name: &str, script: &ScriptInfo) {
    let script_filename = join_scope_id_and_name(script.id.as_str(), script.name.as_str());
    let script_filename = escape_filename(&script_filename);

    // Determine output directory based on element path
    let element_path = script
        .path
        .iter()
        .map(|e| escape_filename(e))
        .collect::<Vec<_>>()
        .join("/");

    let xml_output_dir = dir_path
        .join("scripts")
        .join(fm_file_name)
        .join(&element_path);
    fs::create_dir_all(&xml_output_dir).unwrap_or_else(|err| {
        panic!(
            "Error creating directory {}: {}",
            xml_output_dir.display(),
            err
        )
    });
    write_script_to_xml_file(&xml_output_dir, &script_filename, &script.xml);

    let txt_output_dir = dir_path
        .join("scripts_sanitized")
        .join(fm_file_name)
        .join(&element_path);
    fs::create_dir_all(&txt_output_dir).unwrap_or_else(|err| {
        panic!(
            "Error creating directory {}: {}",
            txt_output_dir.display(),
            err
        )
    });
    write_script_to_text_file(&txt_output_dir, &script_filename, &script.text);
}

fn write_script_to_xml_file(output_dir: &Path, script_filename: &str, content: &str) {
    let output_file_path = output_dir.join(format!("{}.xml", script_filename));
    write_xml_file(&output_file_path, content, 4);
}

fn write_script_to_text_file(output_dir: &Path, script_filename: &str, content: &str) {
    let output_file_path = output_dir.join(format!("{}.txt", script_filename));
    write_text_file(&output_file_path, content);
}
