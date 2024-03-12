use crate::{escape_filename, join_scope_id_and_name};
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::io::{BufRead, Read};
use std::path::Path;

use crate::utils::attributes::get_attributes;
use crate::utils::xml_utils::{
    cdata_element_to_string, end_element_to_string, local_name_to_string, skip_element,
    start_element_to_string, text_element_to_string,
};
use crate::utils::{initialize_out_dir, write_xml_file};

#[derive(Debug, Default)]
struct PrivilegeSet {
    id: String,
    name: String,
    content: String,
}

impl PrivilegeSet {
    pub fn clear(&mut self) {
        self.id.clear();
        self.name.clear();
        self.content.clear();
    }
}

pub fn xml_explode_privilege_set_catalog<R: Read + BufRead>(
    reader: &mut Reader<R>,
    _: &BytesStart,
    out_dir_path: &Path,
    fm_file_name: &str,
) {
    let out_dir_path = out_dir_path.join("privilege_sets").join(fm_file_name);
    initialize_out_dir(&out_dir_path);

    let mut privilege_set = PrivilegeSet::default();

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

                if depth == 3 {
                    if e.name().as_ref() == b"PrivilegeSet" {
                        privilege_set.clear();
                        for attr in get_attributes(&e).unwrap() {
                            match attr.0.as_str() {
                                "id" => privilege_set.id = attr.1.to_string(),
                                "name" => privilege_set.name = attr.1.to_string(),
                                _ => {}
                            }
                        }
                    } else {
                        skip_element(reader, &e);
                        depth -= 1;
                        continue;
                    }
                }

                if privilege_set.id.is_empty() {
                    continue;
                }
                privilege_set
                    .content
                    .push_str(start_element_to_string(&e).as_str());
            }
            Ok(Event::End(e)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }

                if privilege_set.id.is_empty() {
                    continue;
                }

                privilege_set
                    .content
                    .push_str(end_element_to_string(&e).as_str());

                if depth == 2 && local_name_to_string(e.name().as_ref()) == "PrivilegeSet" {
                    write_value_list_to_file(&out_dir_path, &privilege_set);
                    privilege_set.clear();
                }
            }
            Ok(Event::CData(e)) => {
                if privilege_set.id.is_empty() {
                    continue;
                }
                privilege_set
                    .content
                    .push_str(cdata_element_to_string(&e).as_str());
            }
            Ok(Event::Text(e)) | Ok(Event::Comment(e)) => {
                if privilege_set.id.is_empty() {
                    continue;
                }
                privilege_set
                    .content
                    .push_str(text_element_to_string(&e, true).as_str());
            }
            _ => {}
        }

        buf.clear()
    }
}

fn write_value_list_to_file(output_dir: &Path, privilege_set: &PrivilegeSet) {
    let filename = join_scope_id_and_name(privilege_set.id.as_str(), privilege_set.name.as_str());
    let filename = escape_filename(&filename);

    let output_file_path = output_dir.join(format!("{}.xml", filename));
    write_xml_file(&output_file_path, &privilege_set.content, 5);
}
