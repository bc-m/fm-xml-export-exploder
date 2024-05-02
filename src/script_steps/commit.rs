use quick_xml::events::Event;
use quick_xml::Reader;

use crate::utils::attributes::get_attribute;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut suppress_validations = false;
    let mut suppress_validations_text = String::new();
    let mut with_dialog = false;
    let mut with_dialog_text = String::new();
    let mut force = false;
    let mut force_text = String::new();

    let mut reader = Reader::from_str(step);
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
                b"Boolean" => {
                    let id = get_attribute(&e, "id");

                    let mut state = false;
                    match get_attribute(&e, "value").unwrap().as_str() {
                        "True" => {
                            state = true;
                        }
                        "False" => {
                            state = false;
                        }
                        _ => {}
                    }

                    let label = get_attribute(&e, "type").unwrap_or_default().to_string();
                    match id.unwrap().as_str() {
                        "128" => {
                            with_dialog = state;
                            with_dialog_text.push_str(&label);
                        }
                        "256" => {
                            suppress_validations = state;
                            suppress_validations_text.push_str(&label);
                        }
                        "512" => {
                            force = state;
                            force_text.push_str(&label);
                        }
                        _ => {}
                    }
                    continue;
                }
                _ => {}
            },
            _ => {}
        }
        buf.clear()
    }

    if name.is_empty() {
        return None;
    }

    let mut params_text = String::new();
    if suppress_validations {
        params_text.push_str(&format!("{} ; ", suppress_validations_text))
    }
    params_text.push_str(&format!(
        "{}: {}",
        with_dialog_text,
        match with_dialog {
            true => {
                "ON"
            }
            false => {
                "OFF"
            }
        }
    ));
    if force {
        params_text.push_str(&format!(" ; {}", force_text))
    }

    Some(format!("{} [ {} ]", name, params_text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let xml = r#"
            <Step id="75" name="Schreibe Änderung Datens./Abfrage" enable="True">
                <Options>384</Options>
                <ParameterValues membercount="3">
                    <Parameter type="Boolean">
                        <Boolean type="Dateneingabeüberprüfung unterdrücken" id="256" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Mit Dialog" id="128" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Schreiben erzwingen" id="512" value="False"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output =
            Some("Schreibe Änderung Datens./Abfrage [ Mit Dialog: OFF ]".to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn test_force() {
        let xml = r#"
            <Step id="75" name="Schreibe Änderung Datens./Abfrage" enable="True">
                <Options>384</Options>
                <ParameterValues membercount="3">
                    <Parameter type="Boolean">
                        <Boolean type="Dateneingabeüberprüfung unterdrücken" id="256" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Mit Dialog" id="128" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Schreiben erzwingen" id="512" value="True"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(
            "Schreibe Änderung Datens./Abfrage [ Mit Dialog: OFF ; Schreiben erzwingen ]"
                .to_string(),
        );
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn test_suppress_validate() {
        let xml = r#"
            <Step id="75" name="Schreibe Änderung Datens./Abfrage" enable="True">
                <Options>384</Options>
                <ParameterValues membercount="3">
                    <Parameter type="Boolean">
                        <Boolean type="Dateneingabeüberprüfung unterdrücken" id="256" value="True"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Mit Dialog" id="128" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Schreiben erzwingen" id="512" value="False"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Schreibe Änderung Datens./Abfrage [ Dateneingabeüberprüfung unterdrücken ; Mit Dialog: OFF ]".to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn test_all_options() {
        let xml = r#"
            <Step id="75" name="Schreibe Änderung Datens./Abfrage" enable="True">
                <Options>384</Options>
                <ParameterValues membercount="3">
                    <Parameter type="Boolean">
                        <Boolean type="Dateneingabeüberprüfung unterdrücken" id="256" value="True"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Mit Dialog" id="128" value="True"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Schreiben erzwingen" id="512" value="True"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Schreibe Änderung Datens./Abfrage [ Dateneingabeüberprüfung unterdrücken ; Mit Dialog: ON ; Schreiben erzwingen ]".to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }
}
