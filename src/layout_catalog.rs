use crate::{escape_filename, join_scope_id_and_name};
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::fs;
use std::io::{BufRead, Read};
use std::path::Path;

use crate::utils::attributes::get_attributes;
use crate::utils::xml_utils::{
    cdata_element_to_string, end_element_to_string, local_name_to_string, skip_element,
    start_element_to_string, text_element_to_string,
};
use crate::utils::{initialize_out_dir, write_xml_file};

#[derive(Debug, Default)]
struct LayoutInfo {
    id: String,
    name: String,
    content: String,
    path: Vec<String>,
}

pub fn xml_explode_layout_catalog<R: Read + BufRead>(
    reader: &mut Reader<R>,
    _: &BytesStart,
    out_dir_path: &Path,
    fm_file_name: &str,
) {
    let out_dir_path = out_dir_path.join("layouts").join(fm_file_name);
    initialize_out_dir(&out_dir_path);

    let mut layout_info = LayoutInfo::default();

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
                    layout_info.id.clear();
                    layout_info.name.clear();
                    layout_info.content.clear();
                }
                if depth == 2 && e.name().as_ref() != b"Layout" {
                    skip_element(reader, &e);
                    depth -= 1;
                    continue;
                }

                if e.name().as_ref() == b"Layout" {
                    for attr in get_attributes(&e).unwrap() {
                        match attr.0.as_str() {
                            "id" => layout_info.id = attr.1.to_string(),
                            "name" => layout_info.name = attr.1.to_string(),
                            "isFolder" => match attr.1.as_str() {
                                "True" => {
                                    layout_info.path.push(join_scope_id_and_name(
                                        layout_info.id.as_str(),
                                        layout_info.name.as_str(),
                                    ));
                                    skip_element(reader, &e);
                                    depth -= 1;
                                    continue;
                                }
                                "Marker" => {
                                    layout_info.path.pop();
                                    skip_element(reader, &e);
                                    depth -= 1;
                                    continue;
                                }
                                _ => {}
                            },
                            "isSeparatorItem" => {
                                skip_element(reader, &e);
                                depth -= 1;
                                continue;
                            }
                            _ => {}
                        }
                    }
                }

                layout_info
                    .content
                    .push_str(start_element_to_string(&e).as_str());
            }
            Ok(Event::End(e)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }

                layout_info
                    .content
                    .push_str(end_element_to_string(&e).as_str());

                if depth == 1 && local_name_to_string(e.name().as_ref()) == "Layout" {
                    write_layout_to_file(&out_dir_path, &layout_info)
                }
            }
            Ok(Event::CData(e)) => {
                layout_info
                    .content
                    .push_str(cdata_element_to_string(&e).as_str());
            }
            Ok(Event::Text(e)) => {
                layout_info
                    .content
                    .push_str(text_element_to_string(&e, true).as_str());
            }
            Ok(Event::Comment(e)) => {
                layout_info
                    .content
                    .push_str(text_element_to_string(&e, false).as_str());
            }
            _ => {}
        }

        buf.clear()
    }
}

fn write_layout_to_file(dir_path: &Path, layout: &LayoutInfo) {
    let layout_filename = join_scope_id_and_name(layout.id.as_str(), layout.name.as_str());
    let layout_filename = escape_filename(&layout_filename);

    // Determine output directory based on element path
    let element_path = layout
        .path
        .iter()
        .map(|e| escape_filename(e))
        .collect::<Vec<_>>()
        .join("/");

    let output_dir = dir_path.join(element_path);
    fs::create_dir_all(&output_dir)
        .unwrap_or_else(|err| panic!("Error creating directory {}: {}", output_dir.display(), err));

    let output_file_path = output_dir.join(layout_filename).with_extension("xml");
    write_xml_file(&output_file_path, &layout.content, 4);
}
