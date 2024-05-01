use crate::script_steps::parameters::field_reference::FieldReference;
use crate::utils::attributes::get_attribute;
use quick_xml::events::Event;
use quick_xml::Reader;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut select_label = String::new();
    let mut select_enabled = false;
    let mut field_reference = String::new();

    let mut reader = Reader::from_str(step);
    reader.trim_text(true);
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => {
                    name = get_attribute(&e, "name").unwrap().to_string();
                }
                b"Boolean" => {
                    if let Some(name) = get_attribute(&e, "type") {
                        select_label = name.to_string();
                    }

                    match get_attribute(&e, "value").unwrap().as_str() {
                        "True" => {
                            select_enabled = true;
                        }
                        "False" => {
                            select_enabled = false;
                        }
                        _ => {}
                    }
                }
                b"FieldReference" => field_reference.push_str(
                    FieldReference::from_xml(&mut reader, &e)
                        .unwrap()
                        .display()
                        .unwrap()
                        .as_str(),
                ),
                _ => {}
            },
            _ => {}
        }
        buf.clear()
    }

    if name.is_empty() {
        None
    } else {
        let mut parameter = String::new();
        match select_enabled {
            true => parameter.push_str(&select_label),
            false => {}
        }

        if !field_reference.is_empty() {
            if !parameter.is_empty() {
                parameter.push_str(" ; ")
            }
            parameter.push_str(&field_reference.to_string())
        }

        if parameter.is_empty() {
            Some(format!("{} []", name))
        } else {
            Some(format!("{} [ {} ]", name, parameter))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize() {
        let xml_input = "
        <Step index=\"22\" id=\"17\" name=\"Gehe zu Feld\" enable=\"True\">
            <UUID>458FFEBF-72B6-4D92-BCFB-1B2D0D78D456</UUID>
            <OwnerID></OwnerID>
            <Options>0</Options>
            <ParameterValues membercount=\"1\">
                <Parameter type=\"Boolean\">
                    <Boolean type=\"Auswählen/Ausführen\" id=\"4096\" value=\"False\"></Boolean>
                </Parameter>
            </ParameterValues>
        </Step>
        ";

        let expected_output = Some("Gehe zu Feld []".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_sanitize_select() {
        let xml_input = "
        <Step index=\"23\" id=\"17\" name=\"Gehe zu Feld\" enable=\"True\">
            <UUID>C8E7D1E1-9C0C-458F-8835-A1686DEA003C</UUID>
            <OwnerID></OwnerID>
            <Options>4096</Options>
            <ParameterValues membercount=\"1\">
                <Parameter type=\"Boolean\">
                    <Boolean type=\"Auswählen/Ausführen\" id=\"4096\" value=\"True\"></Boolean>
                </Parameter>
            </ParameterValues>
        </Step>
        ";

        let expected_output = Some("Gehe zu Feld [ Auswählen/Ausführen ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_sanitize_with_field_reference() {
        let xml_input = "
        <Step index=\"24\" id=\"17\" name=\"Gehe zu Feld\" enable=\"True\">
            <UUID>226D2607-5BFF-4283-BD95-FBDACC64426D</UUID>
            <OwnerID></OwnerID>
            <Options>4097</Options>
            <ParameterValues membercount=\"2\">
                <Parameter type=\"Boolean\">
                    <Boolean type=\"Auswählen/Ausführen\" id=\"4096\" value=\"True\"></Boolean>
                </Parameter>
                <Parameter type=\"FieldReference\">
                    <FieldReference id=\"1\" name=\"Bar\" UUID=\"09C62F3A-8080-4C3D-A68B-156A166D438E\">
                        <repetition value=\"1\"></repetition>
                        <TableOccurrenceReference id=\"1065090\" name=\"Foo\" UUID=\"EA9E5069-FF8F-41E3-A59E-3530B900EE56\"></TableOccurrenceReference>
                    </FieldReference>
                </Parameter>
            </ParameterValues>
        </Step>
        ";

        let expected_output = Some("Gehe zu Feld [ Auswählen/Ausführen ; Foo::Bar ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }
}
