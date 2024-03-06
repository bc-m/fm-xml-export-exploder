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
struct ValueListInfo {
    id: String,
    name: String,
    content: String,
}

pub fn xml_explode_value_list_catalog<R: Read + BufRead>(
    reader: &mut Reader<R>,
    _: &BytesStart,
    out_dir_path: &Path,
    fm_file_name: &str,
) {
    let out_dir_path = out_dir_path.join("value_lists").join(fm_file_name);
    initialize_out_dir(&out_dir_path);

    let mut value_list_info = ValueListInfo::default();

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
                    value_list_info.id.clear();
                    value_list_info.name.clear();
                    value_list_info.content.clear();
                }
                if depth == 2 && e.name().as_ref() != b"ValueList" {
                    skip_element(reader, &e);
                    depth -= 1;
                    continue;
                }

                if e.name().as_ref() == b"ValueList" {
                    for attr in get_attributes(&e).unwrap() {
                        match attr.0.as_str() {
                            "id" => value_list_info.id = attr.1.to_string(),
                            "name" => value_list_info.name = attr.1.to_string(),
                            _ => {}
                        }
                    }
                }

                value_list_info
                    .content
                    .push_str(start_element_to_string(&e).as_str());
            }
            Ok(Event::End(e)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }

                value_list_info
                    .content
                    .push_str(end_element_to_string(&e).as_str());

                if depth == 1 && local_name_to_string(e.name().as_ref()) == "ValueList" {
                    write_value_list_to_file(&out_dir_path, &value_list_info)
                }
            }
            Ok(Event::CData(e)) => {
                value_list_info
                    .content
                    .push_str(cdata_element_to_string(&e).as_str());
            }
            Ok(Event::Text(e)) => {
                value_list_info
                    .content
                    .push_str(text_element_to_string(&e, true).as_str());
            }
            Ok(Event::Comment(e)) => {
                value_list_info
                    .content
                    .push_str(text_element_to_string(&e, false).as_str());
            }
            _ => {}
        }

        buf.clear()
    }
}

fn write_value_list_to_file(output_dir: &Path, value_list: &ValueListInfo) {
    let value_list_filename =
        join_scope_id_and_name(value_list.id.as_str(), value_list.name.as_str());
    let value_list_filename = escape_filename(&value_list_filename);

    let output_file_path = output_dir.join(format!("{}.xml", value_list_filename));
    write_xml_file(&output_file_path, &value_list.content, 4);
}
