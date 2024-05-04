use std::io::{BufRead, Read};
use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;

use crate::utils::attributes::get_attribute;
use crate::utils::xml_utils::{
    cdata_element_to_string, end_element_to_string, local_name_to_string, skip_element,
    start_element_to_string, text_element_to_string,
};
use crate::utils::{initialize_out_dir, write_xml_file};
use crate::{escape_filename, join_scope_id_and_name};

#[derive(Debug, Default)]
struct RelationshipInfo {
    id: String,
    left: String,
    right: String,
    content: String,
}

pub fn xml_explode_relationship_catalog<R: Read + BufRead>(
    reader: &mut Reader<R>,
    _: &BytesStart,
    out_dir_path: &Path,
    fm_file_name: &str,
) {
    let out_dir_path = out_dir_path.join("relationships").join(fm_file_name);
    initialize_out_dir(&out_dir_path);

    let mut relationship_info = RelationshipInfo::default();
    let mut in_left = false;
    let mut in_right = false;

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
                    if e.name().as_ref() == b"UUID" {
                        skip_element(reader, &e);
                        depth -= 1;
                        continue;
                    }

                    relationship_info.id.clear();
                    relationship_info.left.clear();
                    relationship_info.right.clear();
                    relationship_info.content.clear();

                    if e.name().as_ref() == b"Relationship" {
                        relationship_info.id = get_attribute(&e, "id").unwrap();
                    }
                } else if depth == 3 {
                    match e.name().as_ref() {
                        b"LeftTable" => {
                            in_left = true;
                            relationship_info.left = get_attribute(&e, "name").unwrap_or_default();
                        }
                        b"RightTable" => {
                            in_right = true;
                            relationship_info.right = get_attribute(&e, "name").unwrap_or_default();
                        }
                        _ => {}
                    }
                } else if depth == 4 && e.name().as_ref() == b"TableOccurrenceReference" {
                    if in_left {
                        relationship_info.left = get_attribute(&e, "name").unwrap();
                    } else if in_right {
                        relationship_info.right = get_attribute(&e, "name").unwrap();
                    }
                }

                relationship_info
                    .content
                    .push_str(start_element_to_string(&e).as_str());
            }
            Ok(Event::End(e)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }

                relationship_info
                    .content
                    .push_str(end_element_to_string(&e).as_str());

                if depth == 1 && local_name_to_string(e.name().as_ref()) == "Relationship" {
                    write_relationship_to_file(&out_dir_path, &relationship_info);
                    relationship_info.id.clear();
                    relationship_info.left.clear();
                    relationship_info.right.clear();
                    relationship_info.content.clear();
                } else if depth == 2 {
                    match e.name().as_ref() {
                        b"LeftTable" => {
                            in_left = false;
                        }
                        b"RightTable" => {
                            in_right = false;
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::CData(e)) => {
                relationship_info
                    .content
                    .push_str(cdata_element_to_string(&e).as_str());
            }
            Ok(Event::Text(e)) | Ok(Event::Comment(e)) => {
                if depth < 2 {
                    continue;
                }

                relationship_info
                    .content
                    .push_str(text_element_to_string(&e, true).as_str());
            }
            _ => {}
        }

        buf.clear()
    }
}

fn write_relationship_to_file(output_dir: &Path, relationship: &RelationshipInfo) {
    let relationship_filename = join_scope_id_and_name(
        relationship.id.as_str(),
        format!("[{}] - [{}]", relationship.left, relationship.right).as_str(),
    );
    let relationship_filename = escape_filename(&relationship_filename).replace('.', "_");
    let output_file_path = output_dir.join(format!("{}.xml", relationship_filename));
    write_xml_file(&output_file_path, &relationship.content, 4);
}
