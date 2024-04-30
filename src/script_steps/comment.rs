use crate::utils::attributes::get_attribute;

use quick_xml::escape::unescape;
use quick_xml::events::Event;
use quick_xml::Reader;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut comment = String::new();

    let mut reader = Reader::from_str(step);
    reader.trim_text(true);
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => match get_attribute(&e, "name") {
                    None => {}
                    Some(value) => {
                        name = value.to_string();
                    }
                },
                b"Comment" => {
                    comment = get_attribute(&e, "value").unwrap_or_default();
                }
                _ => {}
            },
            _ => {}
        }
        buf.clear()
    }

    if name.is_empty() {
        None
    } else if comment.is_empty() {
        Some("".to_string())
    } else {
        Some(format!("# {}", unescape(&comment).unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize() {
        let xml_input = "
            <Step id=\"89\" name=\"# (Kommentar)\" enable=\"True\">
                <ParameterValues membercount=\"1\">
                    <ParameterValues membercount=\"1\">
                        <Parameter type=\"Comment\">
                            <Comment value=\"Lorem Ipsum\"></Comment>
                        </Parameter>
                    </ParameterValues>
                </ParameterValues>
            </Step>
        ";

        let expected_output = Some("# Lorem Ipsum".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_sanitize_empty_comment() {
        let xml_input = "
            <Step id=\"89\" name=\"# (Kommentar)\" enable=\"True\">
                <ParameterValues membercount=\"1\">
                    <ParameterValues membercount=\"1\">
                        <Parameter type=\"Comment\">
                            <Comment></Comment>
                        </Parameter>
                    </ParameterValues>
                </ParameterValues>
            </Step>
        ";

        let expected_output = Some("".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }
}
