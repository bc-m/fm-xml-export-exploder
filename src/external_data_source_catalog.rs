use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::io::{BufRead, Read};
use std::path::Path;

use crate::utils::xml_utils::{
    cdata_element_to_string, end_element_to_string, start_element_to_string, text_element_to_string,
};
use crate::utils::{create_dir, write_xml_file};

pub fn xml_extract_external_data_sources<R: Read + BufRead>(
    reader: &mut Reader<R>,
    start: &BytesStart,
    out_dir_path: &Path,
    fm_file_name: &str,
) {
    let out_dir_path = out_dir_path.join("external_data_sources");
    create_dir(&out_dir_path);

    let mut external_data_source_info = String::new();
    external_data_source_info.push_str(start_element_to_string(start).as_str());

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
                external_data_source_info.push_str(start_element_to_string(&e).as_str());
            }
            Ok(Event::End(e)) => {
                depth -= 1;
                external_data_source_info.push_str(end_element_to_string(&e).as_str());

                if depth == 0 {
                    write_external_data_sources_to_file(
                        &out_dir_path,
                        fm_file_name,
                        &external_data_source_info,
                    );
                    break;
                }
            }
            Ok(Event::CData(e)) => {
                external_data_source_info.push_str(cdata_element_to_string(&e).as_str());
            }
            Ok(Event::Text(e)) => {
                external_data_source_info.push_str(text_element_to_string(&e, true).as_str());
            }
            Ok(Event::Comment(e)) => {
                external_data_source_info.push_str(text_element_to_string(&e, false).as_str());
            }
            _ => {}
        }

        buf.clear()
    }
}

fn write_external_data_sources_to_file(output_dir: &Path, fm_file_name: &str, content: &str) {
    let output_file_path = output_dir.join(fm_file_name).with_extension("xml");
    write_xml_file(&output_file_path, content, 3);
}
