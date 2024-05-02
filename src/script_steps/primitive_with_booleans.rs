use quick_xml::events::Event;
use quick_xml::Reader;

use crate::script_steps::parameters::parameter_values::ParameterValues;
use crate::utils::attributes::get_attribute;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut parameters: Vec<String> = Vec::new();

    let mut reader = Reader::from_str(step);
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => {
                    match get_attribute(&e, "name") {
                        None => {}
                        Some(value) => {
                            name = value.to_string();
                        }
                    }
                    continue;
                }
                b"ParameterValues" => parameters.push(
                    ParameterValues::from_xml(&mut reader, &e)
                        .unwrap()
                        .display()
                        .unwrap(),
                ),
                _ => {}
            },
            _ => {}
        }
        buf.clear()
    }

    if parameters.is_empty() {
        return Some(format!("{} []", name));
    };

    Some(format!("{} [ {} ]", name, parameters.join(" ; ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enabled() {
        let xml = r#"
            <Step id="86" name="Fehleraufzeichnung setzen" enable="True">
                <Options>196608</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Boolean">
                        <Boolean id="131072" value="True"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Fehleraufzeichnung setzen [ ON ]".to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn test_disabled() {
        let xml = r#"
            <Step id="86" name="Fehleraufzeichnung setzen" enable="True">
                <Options>196608</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Boolean">
                        <Boolean id="131072" value="False"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Fehleraufzeichnung setzen [ OFF ]".to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn test_enter_find_mode() {
        let xml = r#"
            <Step id="22" name="Suchenmodus aktivieren" enable="True">
                <SourceUUID>3C5EBF9C-BAE0-460F-92AF-3FB73649951B</SourceUUID>
                <ParameterValues membercount="1">
                    <Parameter type="Boolean">
                        <Boolean type="Pause" id="16777216" value="False"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Suchenmodus aktivieren [ Pause: OFF ]".to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }
}
