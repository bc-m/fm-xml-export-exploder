use crate::{escape_filename, join_scope_id_and_name};
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::io::{BufRead, Read};
use std::path::Path;

use crate::utils::attributes::get_attribute;
use crate::utils::xml_utils::{
    cdata_element_to_string, end_element_to_string, local_name_to_string, skip_element,
    start_element_to_string, text_element_to_string,
};
use crate::utils::{initialize_out_dir, write_xml_file};

#[derive(Debug, Default)]
struct TableOccurrenceInfo {
    id: String,
    name: String,
    content: String,
}

pub fn xml_explode_table_occurrence_catalog<R: Read + BufRead>(
    reader: &mut Reader<R>,
    _: &BytesStart,
    out_dir_path: &Path,
    fm_file_name: &str,
) {
    let out_dir_path = out_dir_path.join("table_occurrences").join(fm_file_name);
    initialize_out_dir(&out_dir_path);

    let mut table_occurrence_info = TableOccurrenceInfo::default();

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
                    table_occurrence_info.id.clear();
                    table_occurrence_info.name.clear();
                    table_occurrence_info.content.clear();

                    if e.name().as_ref() == b"TableOccurrence" {
                        table_occurrence_info.id = get_attribute(&e, "id").unwrap();
                        table_occurrence_info.name = get_attribute(&e, "name").unwrap();
                    } else {
                        skip_element(reader, &e);
                        depth -= 1;
                        continue;
                    };
                }

                table_occurrence_info
                    .content
                    .push_str(start_element_to_string(&e).as_str());
            }
            Ok(Event::End(e)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }

                table_occurrence_info
                    .content
                    .push_str(end_element_to_string(&e).as_str());

                if depth == 1 && local_name_to_string(e.name().as_ref()) == "TableOccurrence" {
                    write_table_occurrence_to_file(&out_dir_path, &table_occurrence_info);
                    table_occurrence_info.id.clear();
                    table_occurrence_info.name.clear();
                    table_occurrence_info.content.clear();
                }
            }
            Ok(Event::CData(e)) => {
                table_occurrence_info
                    .content
                    .push_str(cdata_element_to_string(&e).as_str());
            }
            Ok(Event::Text(e)) => {
                table_occurrence_info
                    .content
                    .push_str(text_element_to_string(&e, true).as_str());
            }
            Ok(Event::Comment(e)) => {
                table_occurrence_info
                    .content
                    .push_str(text_element_to_string(&e, false).as_str());
            }
            _ => {}
        }

        buf.clear()
    }
}

fn write_table_occurrence_to_file(output_dir: &Path, table_occurrence: &TableOccurrenceInfo) {
    let filename =
        join_scope_id_and_name(table_occurrence.id.as_str(), table_occurrence.name.as_str());
    let filename = escape_filename(&filename).replace('.', "_");
    let output_file_path = output_dir.join(filename).with_extension("xml");
    write_xml_file(&output_file_path, &table_occurrence.content, 4);
}
