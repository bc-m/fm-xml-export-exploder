use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::io::{BufRead, Read};
use std::path::Path;

use crate::utils::xml_utils::skip_element;
use crate::utils::{initialize_out_dir, write_xml_element_to_file};

pub fn xml_explode_custom_menu_catalog<R: Read + BufRead>(
    reader: &mut Reader<R>,
    _: &BytesStart,
    out_dir_path: &Path,
    fm_file_name: &str,
) {
    let out_dir_path = out_dir_path.join("custom_menu").join(fm_file_name);
    initialize_out_dir(&out_dir_path);

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
                    if e.name().as_ref() == b"CustomMenu" {
                        write_xml_element_to_file(reader, &e, &out_dir_path, 4);
                        depth -= 1;
                        continue;
                    } else {
                        skip_element(reader, &e);
                        depth -= 1;
                        continue;
                    }
                }
            }
            Ok(Event::End(_)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }

        buf.clear()
    }
}
