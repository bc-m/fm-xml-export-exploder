use crate::{escape_filename, join_scope_id_and_name};
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::io::{BufRead, Read};
use std::path::Path;

use crate::utils::attributes::get_attributes;
use crate::utils::xml_utils::{cdata_to_string, local_name_to_string};
use crate::utils::{initialize_out_dir, write_text_file};

#[derive(Debug, Default)]
struct CustomFunctionInfo {
    id: String,
    name: String,
    content: String,
}

pub fn xml_explode_custom_function_catalog<R: Read + BufRead>(
    reader: &mut Reader<R>,
    _: &BytesStart,
    out_dir_path: &Path,
    fm_file_name: &str,
) {
    let out_dir_path = out_dir_path.join("custom_functions").join(fm_file_name);
    initialize_out_dir(&out_dir_path);

    let mut custom_function_info = CustomFunctionInfo::default();

    let mut depth = 1;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => {
                break;
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                depth += 1;
                if depth < 3 {
                    continue;
                } else if depth == 3 {
                    custom_function_info.id.clear();
                    custom_function_info.name.clear();
                    custom_function_info.content.clear();
                } else if depth == 4 && e.name().as_ref() == b"CustomFunctionReference" {
                    for attr in get_attributes(&e).unwrap() {
                        match attr.0.as_str() {
                            "id" => custom_function_info.id = attr.1.to_string(),
                            "name" => custom_function_info.name = attr.1.to_string(),
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::End(e)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                } else if depth < 2 {
                    continue;
                } else if depth == 2
                    && local_name_to_string(e.name().as_ref()) == "CustomFunctionCalc"
                {
                    write_custom_function_to_file(&out_dir_path, &custom_function_info);
                    custom_function_info.id.clear();
                    custom_function_info.name.clear();
                    custom_function_info.content.clear();
                }
            }
            Ok(Event::CData(e)) => {
                if depth < 3 {
                    continue;
                }

                custom_function_info
                    .content
                    .push_str(cdata_to_string(&e).as_str());
            }
            _ => {}
        }

        buf.clear()
    }
}

fn write_custom_function_to_file(output_dir: &Path, cf: &CustomFunctionInfo) {
    let cf_filename = join_scope_id_and_name(cf.id.as_str(), cf.name.as_str());
    let cf_filename = escape_filename(&cf_filename);

    let output_file_path = output_dir.join(cf_filename).with_extension("txt");
    write_text_file(&output_file_path, &cf.content);
}
