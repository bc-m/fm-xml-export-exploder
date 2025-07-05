use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use anyhow::Result;
use quick_xml::events::{BytesCData, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader;

use crate::utils::attributes::get_attributes;
use crate::utils::push_line_to_skeleton;
use crate::xml_processor::ProcessingContext;
use crate::{Flags, Skeleton};

#[derive(Debug, Default, PartialEq)]
pub enum XmlEventType {
    #[default]
    Other,
    Start,
    End,
    Text,
    Comment,
    CData,
}

// pub fn event_type_of(event: &Event) -> XmlEventType {

pub fn start_element_to_string(e: &BytesStart, flags: &Flags) -> String {
    let mut complete_tag = String::with_capacity(128); // Pre-allocate
    complete_tag.push('<');
    complete_tag.push_str(&local_name_to_string(e.name().as_ref()));

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
        complete_tag.push(' ');
        complete_tag.push_str(&attr.0);
        complete_tag.push_str("=\"");
        complete_tag.push_str(&attr.1);
        complete_tag.push('"');
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
    format!("</{element_name}>")
}

/// Derive end tag from start tag
pub fn end_element_to_string_from_start_element(e: &BytesStart) -> String {
    let element_name = local_name_to_string(e.name().as_ref());
    format!("</{element_name}>")
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

pub fn encode_xml_special_characters(input: String) -> String {
    input
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&apos;", "'")
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

pub fn skip_rest_of_element<R: Read + BufRead>(reader: &mut Reader<R>, _: &BytesStart) {
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

pub fn push_rest_of_element_to_skeleton<R: Read + BufRead>(
    reader: &mut Reader<R>,
    _: &BytesStart,
    skeleton: &mut Skeleton,
    base_depth: usize,
    flags: &Flags,
) {
    let mut depth = 1;
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                push_line_to_skeleton(
                    skeleton,
                    base_depth,
                    depth,
                    start_element_to_string(&e, flags).as_str(),
                    false,
                    XmlEventType::Start,
                );
                depth += 1;
            }
            Ok(Event::End(e)) => {
                depth -= 1;
                push_line_to_skeleton(
                    skeleton,
                    base_depth,
                    depth,
                    end_element_to_string(&e).as_str(),
                    false,
                    XmlEventType::End,
                );
                if depth == 0 {
                    break;
                }
            }
            Ok(Event::CData(e)) => {
                push_line_to_skeleton(
                    skeleton,
                    base_depth,
                    depth,
                    cdata_element_to_string(&e).as_str(),
                    false,
                    XmlEventType::CData,
                );
            }
            Ok(Event::Text(e)) => {
                if text_element_to_string(&e, true).trim().is_empty() {
                    continue;
                }
                push_line_to_skeleton(
                    skeleton,
                    base_depth,
                    depth,
                    text_element_to_string(&e, true).as_str(),
                    false,
                    XmlEventType::Text,
                );
            }
            Ok(Event::Comment(e)) => {
                push_line_to_skeleton(
                    skeleton,
                    base_depth,
                    depth,
                    text_element_to_string(&e, true).as_str(),
                    false,
                    XmlEventType::Comment,
                );
            }
            _ => {}
        }
        buf.clear();
    }
}

pub fn element_to_string<R: Read + BufRead>(
    context: &mut ProcessingContext<'_, R>,
    start_tag: &BytesStart,
) -> String {
    let mut content = start_element_to_string(start_tag, context.flags);
    let mut depth = 1;
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match context.reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                depth += 1;
                content.push_str(&start_element_to_string(&e, context.flags))
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

/// Extract content from XML file using multiple XPath-like expressions
///
/// # Arguments
/// * `file_path` - Path to the XML file
/// * `xml_paths` - Array of XPath-like expressions
///
/// # Returns
/// * `Ok(Vec<Option<String>>)` - Array of results, one for each path (None if path doesn't exist)
/// * `Err(String)` - If there's an error reading the file or parsing XML
///
/// # Examples
/// ```
/// let paths = vec![
///     "Account/Authentication/AccountName",
///     "Account/@id"
/// ];
/// let results = extract_values_from_xml_paths("file.xml", &paths);
/// assert_eq!(results.unwrap(), vec![Some("[Guest]".to_string()), Some("1".to_string())]);
/// ```
pub fn extract_values_from_xml_paths(
    file_path: &Path,
    target_paths: &[&str],
) -> Result<Vec<Option<String>>, String> {
    let file = File::open(file_path).map_err(|e| format!("Failed to open file: {e}"))?;
    let reader = BufReader::new(file);
    let mut reader = Reader::from_reader(reader);
    reader.config_mut().trim_text(true);

    // Preprocess paths: split each path into components
    let parsed_paths: Vec<Vec<&str>> = target_paths
        .iter()
        .map(|p| p.split('/').collect())
        .collect();

    let mut buf = Vec::new();
    let mut current_path = Vec::new();
    let mut results: Vec<Option<String>> = vec![None; parsed_paths.len()];
    let mut resolved_indices = HashSet::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                current_path.push(tag.clone());

                for (i, path) in parsed_paths.iter().enumerate() {
                    if resolved_indices.contains(&i) {
                        continue;
                    }

                    if let Some(last_segment) = path.last() {
                        if last_segment.starts_with('@') && path.len() == current_path.len() + 1 {
                            let attr_name = &last_segment[1..];
                            if path[..path.len() - 1]
                                .iter()
                                .copied()
                                .eq(current_path.iter().map(|s| s.as_str()))
                            {
                                if let Some(attr) = e
                                    .attributes()
                                    .flatten()
                                    .find(|a| a.key.as_ref() == attr_name.as_bytes())
                                {
                                    let value = attr
                                        .unescape_value()
                                        .map_err(|e| format!("Unescape error: {e}"))?;
                                    if path.starts_with(&["Relationship", "LeftTable"])
                                        || path.starts_with(&["Relationship", "RightTable"])
                                    {
                                        results[i] = Some(format!("[{value}]"));
                                    } else {
                                        results[i] = Some(value.to_string());
                                    }

                                    resolved_indices.insert(i);
                                }
                            }
                        }
                    }
                }
            }

            Ok(Event::End(_)) => {
                current_path.pop();
            }

            Ok(Event::Text(e)) => {
                for (i, path) in parsed_paths.iter().enumerate() {
                    if resolved_indices.contains(&i) {
                        continue;
                    }

                    if let Some(last_segment) = path.last() {
                        if !last_segment.starts_with('@')
                            && path.len() == current_path.len()
                            && path.iter().zip(&current_path).all(|(a, b)| *a == b)
                        {
                            match e.unescape() {
                                Ok(text) => {
                                    results[i] = Some(text.to_string());
                                    resolved_indices.insert(i);
                                }
                                Err(err) => {
                                    return Err(format!("Failed to unescape text: {err}"));
                                }
                            }
                        }
                    }
                }
            }

            Ok(Event::CData(e)) => {
                for (i, path) in parsed_paths.iter().enumerate() {
                    if resolved_indices.contains(&i) {
                        continue;
                    }

                    if let Some(last_segment) = path.last() {
                        if !last_segment.starts_with('@')
                            && path.len() == current_path.len()
                            && path.iter().zip(&current_path).all(|(a, b)| *a == b)
                        {
                            let text = String::from_utf8_lossy(e.as_ref()).to_string();
                            results[i] = Some(text);
                            resolved_indices.insert(i);
                        }
                    }
                }
            }

            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Error parsing XML: {e}")),
            _ => {}
        }

        buf.clear();
    }

    Ok(results)
}
