use crate::{escape_filename, join_scope_id_and_name};
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::io::{BufRead, Read};
use std::path::Path;

use crate::utils::attributes::get_attributes;
use crate::utils::xml_utils::{
    cdata_element_to_string, end_element_to_string, local_name_to_string, start_element_to_string,
    text_element_to_string,
};
use crate::utils::{initialize_out_dir, write_xml_file};

#[derive(Debug, Default)]
struct TableInfo {
    id: String,
    name: String,
    content: String,
}

pub fn xml_explode_table_catalog<R: Read + BufRead>(
    reader: &mut Reader<R>,
    _: &BytesStart,
    out_dir_path: &Path,
    fm_file_name: &str,
) {
    let out_dir_path = out_dir_path.join("tables").join(fm_file_name);
    initialize_out_dir(&out_dir_path);

    let mut table_info = TableInfo::default();

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
                    table_info.id.clear();
                    table_info.name.clear();
                    table_info.content.clear();
                }

                if depth == 3 && e.name().as_ref() == b"BaseTableReference" {
                    for attr in get_attributes(&e).unwrap() {
                        match attr.0.as_str() {
                            "id" => table_info.id = attr.1.to_string(),
                            "name" => table_info.name = attr.1.to_string(),
                            _ => {}
                        }
                    }
                }

                table_info
                    .content
                    .push_str(start_element_to_string(&e).as_str());
            }
            Ok(Event::End(e)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }

                table_info
                    .content
                    .push_str(end_element_to_string(&e).as_str());

                if depth == 1 && local_name_to_string(e.name().as_ref()) == "FieldCatalog" {
                    write_table_to_file(&out_dir_path, &table_info);
                    table_info.id.clear();
                    table_info.name.clear();
                    table_info.content.clear();
                }
            }
            Ok(Event::CData(e)) => {
                table_info
                    .content
                    .push_str(cdata_element_to_string(&e).as_str());
            }
            Ok(Event::Text(e)) => {
                table_info
                    .content
                    .push_str(text_element_to_string(&e, true).as_str());
            }
            Ok(Event::Comment(e)) => {
                table_info
                    .content
                    .push_str(text_element_to_string(&e, false).as_str());
            }
            _ => {}
        }

        buf.clear()
    }
}

fn write_table_to_file(output_dir: &Path, table: &TableInfo) {
    let table_filename = join_scope_id_and_name(table.id.as_str(), table.name.as_str());
    let table_filename = escape_filename(&table_filename);
    let output_file_path = output_dir.join(table_filename).with_extension("xml");
    write_xml_file(&output_file_path, &table.content, 4);
}
