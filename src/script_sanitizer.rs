use std::fs;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::config::Flags;
use crate::script_steps::constants::{ScriptStep, id_to_script_step};
use crate::script_steps::sanitizer::sanitize;
use crate::utils::attributes::get_attribute;
use crate::utils::write_text_file;
use crate::utils::xml_utils::{
    cdata_element_to_string, end_element_to_string, general_ref_to_string, start_element_to_string,
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
    flags: &Flags,
) {
    for_each_xml_file(
        scripts_xml_out_dir_path,
        scripts_xml_out_dir_path,
        scripts_text_out_dir_path,
        &mut |xml_file_path, output_file_path| {
            let Ok(xml_content) = fs::read_to_string(xml_file_path) else {
                eprintln!("Error reading file {}", xml_file_path.display());
                return;
            };
            if let Some(script_info) = parse_script_xml(&xml_content, flags) {
                write_text_file(output_file_path, &script_info.text);
            }
        },
    );
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

                if depth == 2 && e.name().as_ref() == b"ScriptReference" {
                    // Extract script ID and name from ScriptReference
                    for attr in crate::utils::attributes::get_attributes(&e) {
                        match attr.0.as_str() {
                            "id" => script_info.id = attr.1,
                            "name" => script_info.name = attr.1,
                            _ => {}
                        }
                    }
                } else if depth == 3 && e.name().as_ref() == b"Step" {
                    in_step = true;
                    step_info.indent_level_current = step_info.indent_level_next;
                    step_info.id = get_attribute(&e, "id").unwrap().parse::<u32>().unwrap();

                    if get_attribute(&e, "enable").is_none_or(|v| v == "True") {
                        match id_to_script_step(step_info.id) {
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
                        .push_str(&start_element_to_string(&e, flags));
                }
            }
            Ok(Event::End(e)) => {
                depth -= 1;

                if depth == 0 {
                    break;
                }

                if in_step {
                    step_info.content.push_str(&end_element_to_string(&e));
                }

                if depth == 2 && e.name().as_ref() == b"Step" {
                    in_step = false;
                    let is_comment = id_to_script_step(step_info.id) == ScriptStep::Comment;
                    if let Some(text) = sanitize(step_info.id, &step_info.content) {
                        for (i, line) in text.split('\r').enumerate() {
                            let extra_indent = if i > 0 && !is_comment { 4 } else { 0 };
                            let mut indent =
                                "\t".repeat(step_info.indent_level_current + extra_indent);
                            if is_comment && i > 0 {
                                indent.push_str("  ");
                            }
                            script_info.text.push_str(&format!("{indent}{line}\n"));
                        }
                    }
                    step_info.content.clear()
                }
            }
            Ok(Event::CData(e)) => {
                if in_step {
                    step_info.content.push_str(&cdata_element_to_string(&e));
                }
            }
            Ok(Event::Comment(e)) | Ok(Event::Text(e)) => {
                if in_step {
                    step_info
                        .content
                        .push_str(&text_element_to_string(&e, true));
                }
            }
            Ok(Event::GeneralRef(e)) => {
                if in_step {
                    step_info.content.push_str(&general_ref_to_string(&e, true));
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
