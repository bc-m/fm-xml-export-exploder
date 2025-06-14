use std::io::{BufRead, Read};

use quick_xml::events::{BytesCData, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader;

use crate::utils::attributes::get_attributes;
use crate::Flags;

pub fn start_element_to_string(e: &BytesStart, flags: &Flags) -> String {
    let mut complete_tag = format!("<{}", local_name_to_string(e.name().as_ref()));
    for attr in get_attributes(e).unwrap() {
        // de-noise
        if !flags.lossless {
            match attr.0.as_str() {
                "nextvalue" | "UUID" | "index" => {
                    continue;
                }
                _ => {}
            }
        }

        complete_tag.push_str(&format!(" {}=\"{}\"", attr.0, attr.1));
    }
    complete_tag.push('>');

    complete_tag
}

pub fn cdata_element_to_string(e: &BytesCData) -> String {
    let mut content = String::new();
    content.push_str("<![CDATA[");
    content.push_str(cdata_to_string(e).as_str());
    content.push_str("]]>");

    content
}

pub fn text_element_to_string(e: &BytesText, escape: bool) -> String {
    if escape {
        decode_xml_special_characters(text_to_string(e))
    } else {
        text_to_string(e)
    }
}

pub fn end_element_to_string(e: &BytesEnd) -> String {
    let element_name = local_name_to_string(e.name().as_ref());
    format!("</{}>", element_name)
}

pub fn local_name_to_string(local_name: &[u8]) -> String {
    match std::str::from_utf8(local_name) {
        Ok(text) => text.to_string(),
        Err(_) => String::new(),
    }
}

pub fn text_to_string(e: &BytesText) -> String {
    match e.unescape() {
        Ok(text) => text.to_string(),
        Err(_) => String::new(),
    }
}

pub fn cdata_to_string(e: &BytesCData) -> String {
    match std::str::from_utf8(e) {
        Ok(text) => text.to_string(),
        Err(_) => String::new(),
    }
}

fn decode_xml_special_characters(input: String) -> String {
    input
        .replace('&', "&amp;")
        .replace('\"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
        .replace('\r', "&#13;")
}

pub fn skip_element<R: Read + BufRead>(reader: &mut Reader<R>, _: &BytesStart) {
    let mut depth = 1;
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(_)) => {
                depth += 1;
            }
            Ok(Event::End(_)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        buf.clear();
    }
}

pub fn element_to_string<R: Read + BufRead>(
    reader: &mut Reader<R>,
    start: &BytesStart,
    flags: &Flags,
) -> String {
    let mut content = start_element_to_string(start, flags);
    let mut depth = 1;
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                depth += 1;
                content.push_str(&start_element_to_string(&e, flags))
            }
            Ok(Event::CData(e)) => {
                content.push_str(&cdata_element_to_string(&e));
            }
            Ok(Event::Text(e)) | Ok(Event::Comment(e)) => {
                content.push_str(&text_element_to_string(&e, true));
            }
            Ok(Event::End(e)) => {
                content.push_str(&end_element_to_string(&e));
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        buf.clear();
    }

    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_xml_special_characters() {
        assert_eq!(
            decode_xml_special_characters("This & that \"test\" <tag>".to_string()),
            "This &amp; that &quot;test&quot; &lt;tag&gt;"
        );
    }
}
