use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use quick_xml::Reader;
use quick_xml::events::{BytesCData, BytesEnd, BytesRef, BytesStart, BytesText, Event};

use crate::Skeleton;
use crate::config::Flags;
use crate::utils::attributes::get_attributes;
use crate::utils::skeleton::push_line_to_skeleton;
use crate::xml_processor::ProcessingContext;

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

pub fn start_element_to_string(e: &BytesStart, flags: &Flags) -> String {
    let mut complete_tag = String::with_capacity(128); // Pre-allocate
    complete_tag.push('<');
    complete_tag.push_str(&local_name_to_string(e.name().as_ref()));

    for attr in get_attributes(e) {
        if !flags.lossless
            && matches!(
                attr.0.as_str(),
                "nextvalue" | "UUID" | "index" | "Collapsed"
            )
        {
            continue;
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
    format!("<![CDATA[{}]]>", cdata_to_string(e))
}

pub fn text_element_to_string(e: &BytesText, escape: bool) -> String {
    let text = text_to_string(e);
    if escape {
        escape_xml_entities(text)
    } else {
        text
    }
}

pub fn end_element_to_string(e: &BytesEnd) -> String {
    format_end_tag(e.name().as_ref())
}

pub fn format_end_tag(name: &[u8]) -> String {
    let element_name = local_name_to_string(name);
    format!("</{element_name}>")
}

pub fn local_name_to_string(local_name: &[u8]) -> String {
    String::from_utf8_lossy(local_name).into_owned()
}

pub fn text_to_string(e: &BytesText) -> String {
    e.decode().map(|s| s.into_owned()).unwrap_or_default()
}

pub fn cdata_to_string(e: &BytesCData) -> String {
    String::from_utf8_lossy(e).into_owned()
}

/// Convert a general entity reference back to its escaped XML form
/// e.g., BytesRef containing "quot" -> "&quot;", BytesRef containing "#09" -> "&#09;"
pub fn general_ref_to_string(e: &BytesRef, escape: bool) -> String {
    let Ok(entity_name) = e.decode() else {
        return String::new();
    };

    if escape {
        // Keep the reference in its escaped form (e.g., &quot;, &#09;)
        return format!("&{entity_name};");
    }

    // Resolve the reference to its actual character
    // First try character references (e.g., &#65; or &#x41;)
    if let Ok(Some(ch)) = e.resolve_char_ref() {
        return ch.to_string();
    }

    // For named entity references, resolve using unescape
    let escaped = format!("&{entity_name};");
    quick_xml::escape::unescape(&escaped)
        .map(|s| s.to_string())
        .unwrap_or(escaped)
}

pub fn unescape_xml_entities(input: String) -> String {
    input
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&apos;", "'")
}

fn escape_xml_entities(input: String) -> String {
    input
        .replace('&', "&amp;")
        .replace('\"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
        .replace('\r', "&#13;")
}

pub fn skip_rest_of_element<R: Read + BufRead>(reader: &mut Reader<R>) {
    let mut depth = 1;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(_)) => depth += 1,
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
    skeleton: &mut Skeleton,
    base_depth: usize,
    flags: &Flags,
) {
    let mut depth = 1;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                push_line_to_skeleton(
                    skeleton,
                    base_depth,
                    depth,
                    &start_element_to_string(&e, flags),
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
                    &end_element_to_string(&e),
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
                    &cdata_element_to_string(&e),
                    false,
                    XmlEventType::CData,
                );
            }
            Ok(Event::Text(e)) => {
                let text = text_element_to_string(&e, true);
                if text.trim().is_empty() {
                    continue;
                }
                push_line_to_skeleton(
                    skeleton,
                    base_depth,
                    depth,
                    &text,
                    false,
                    XmlEventType::Text,
                );
            }
            Ok(Event::GeneralRef(e)) => {
                push_line_to_skeleton(
                    skeleton,
                    base_depth,
                    depth,
                    &general_ref_to_string(&e, true),
                    false,
                    XmlEventType::Text,
                );
            }
            Ok(Event::Comment(e)) => {
                push_line_to_skeleton(
                    skeleton,
                    base_depth,
                    depth,
                    &text_element_to_string(&e, true),
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
    let mut buf = Vec::new();
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
            Ok(Event::GeneralRef(e)) => {
                content.push_str(&general_ref_to_string(&e, true));
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
    let mut current_path: Vec<String> = Vec::new();
    let mut results: Vec<Option<String>> = vec![None; parsed_paths.len()];
    let mut resolved_indices = HashSet::new();

    /// Returns indices of unresolved text-element paths that match the current path
    fn matching_text_paths(
        parsed_paths: &[Vec<&str>],
        current_path: &[String],
        resolved_indices: &HashSet<usize>,
    ) -> Vec<usize> {
        parsed_paths
            .iter()
            .enumerate()
            .filter(|(i, path)| {
                !resolved_indices.contains(i)
                    && path.last().is_some_and(|last| !last.starts_with('@'))
                    && path.len() == current_path.len()
                    && path.iter().zip(current_path).all(|(a, b)| *a == b)
            })
            .map(|(i, _)| i)
            .collect()
    }

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                current_path.push(String::from_utf8_lossy(e.name().as_ref()).into_owned());

                for (i, path) in parsed_paths.iter().enumerate() {
                    if resolved_indices.contains(&i) {
                        continue;
                    }

                    if let Some(last_segment) = path.last()
                        && last_segment.starts_with('@')
                        && path.len() == current_path.len() + 1
                        && path[..path.len() - 1]
                            .iter()
                            .copied()
                            .eq(current_path.iter().map(|s| s.as_str()))
                    {
                        let attr_name = &last_segment[1..];
                        if let Some(attr) = e
                            .attributes()
                            .flatten()
                            .find(|a| a.key.as_ref() == attr_name.as_bytes())
                        {
                            let value = attr
                                .unescape_value()
                                .map_err(|e| format!("Unescape error: {e}"))?;
                            results[i] = if path.starts_with(&["Relationship", "LeftTable"])
                                || path.starts_with(&["Relationship", "RightTable"])
                            {
                                Some(format!("[{value}]"))
                            } else {
                                Some(value.to_string())
                            };
                            resolved_indices.insert(i);
                        }
                    }
                }
            }

            Ok(Event::End(_)) => {
                for i in matching_text_paths(&parsed_paths, &current_path, &resolved_indices) {
                    if results[i].is_some() {
                        resolved_indices.insert(i);
                    }
                }
                current_path.pop();
            }

            Ok(Event::Text(e)) => {
                for i in matching_text_paths(&parsed_paths, &current_path, &resolved_indices) {
                    let decoded = e
                        .decode()
                        .map_err(|err| format!("Failed to decode text: {err}"))?;
                    results[i]
                        .get_or_insert_with(String::new)
                        .push_str(&decoded);
                }
            }

            Ok(Event::GeneralRef(e)) => {
                let resolved = general_ref_to_string(&e, false);
                for i in matching_text_paths(&parsed_paths, &current_path, &resolved_indices) {
                    results[i]
                        .get_or_insert_with(String::new)
                        .push_str(&resolved);
                }
            }

            Ok(Event::CData(e)) => {
                for i in matching_text_paths(&parsed_paths, &current_path, &resolved_indices) {
                    results[i] = Some(String::from_utf8_lossy(e.as_ref()).into_owned());
                    resolved_indices.insert(i);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_xml_entities() {
        assert_eq!(
            escape_xml_entities("This & that \"test\" <tag>".to_string()),
            "This &amp; that &quot;test&quot; &lt;tag&gt;"
        );
    }
}
