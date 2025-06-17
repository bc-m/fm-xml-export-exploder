use std::io::{BufRead, Read};

use quick_xml::events::{BytesStart, Event};

use crate::utils::xml_utils::{
    cdata_element_to_string, end_element_to_string, start_element_to_string,
    text_element_to_string, XmlEventType,
};
use crate::utils::{push_line_to_skeleton, write_xml_file};
use crate::xml_processor::ProcessingContext;

/// Extract entire element as is into its own file (don't split it into multiple files)
/// Text content inside of <Chunk> tags needs special handling (replace tabs with &#09;)
/// Also generates skeleton XML if lossless mode is enabled
pub fn process_supporting_element<R: Read + BufRead>(
    context: &mut ProcessingContext<'_, R>,
    start_tag: &BytesStart,
    out_file_name: &str,
) {
    let out_file_path = context
        .current_out_dir
        .join(format!("{}.xml", out_file_name));

    let mut ddr_info = String::new();
    ddr_info.push_str(start_element_to_string(start_tag, context.flags).as_str());

    let mut depth = 1; // 1 means DDR_INFO
    let base_depth = context.path_stack.len();
    let mut buf = Vec::new();
    let mut in_chunk = false;
    loop {
        match context.reader.read_event_into(&mut buf) {
            Err(e) => {
                println!("Error {}", e);
                break;
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                depth += 1;
                let start_tag = start_element_to_string(&e, context.flags);
                if e.name().as_ref() == b"Chunk" {
                    in_chunk = true;
                }

                // Add to skeleton
                if context.flags.lossless && depth == 1 {
                    push_line_to_skeleton(
                        context.skeleton,
                        base_depth,
                        depth,
                        &start_tag,
                        false,
                        XmlEventType::Start,
                    );
                }

                ddr_info.push_str(&start_tag);
            }
            Ok(Event::End(e)) => {
                depth -= 1;
                let end_tag = end_element_to_string(&e);

                if context.flags.lossless && depth == 0 {
                    push_line_to_skeleton(
                        context.skeleton,
                        base_depth,
                        depth,
                        &end_tag,
                        false,
                        XmlEventType::End,
                    );
                }

                ddr_info.push_str(&end_tag);

                if in_chunk && e.name().as_ref() == b"Chunk" {
                    in_chunk = false;
                }

                // Write to file if we're at the end of the catalog
                if depth == 0 {
                    write_xml_file(&out_file_path, &ddr_info, 1, context.flags);
                    context.path_stack.pop();
                    break;
                }
            }
            Ok(Event::CData(e)) => {
                ddr_info.push_str(cdata_element_to_string(&e).as_str());
            }
            Ok(Event::Text(e)) => {
                let mut text_string = text_element_to_string(&e, true);
                if in_chunk && !text_string.trim().is_empty() {
                    text_string = text_string.replace('\t', "&#09;");
                }
                ddr_info.push_str(text_string.as_str());
            }
            Ok(Event::Comment(e)) => {
                ddr_info.push_str(text_element_to_string(&e, false).as_str());
            }
            _ => {}
        }

        buf.clear()
    }
}
