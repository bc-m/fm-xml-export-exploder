use crate::utils::attributes::get_attribute;
use quick_xml::events::Event;
use quick_xml::Reader;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();

    let mut restore = false;

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
                b"ParameterValues" => {
                    restore = true;
                    break;
                }
                _ => {}
            },
            _ => {}
        }
        buf.clear()
    }

    if name.is_empty() {
        println!("empty primitive");
        None
    } else {
        match restore {
            true => Some(format!("{} [ ⚠️ RESTORE ⚠️ ]", name)),
            false => Some(name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let xml_input = "
        <Step id=\"28\" name=\"Ergebnismenge suchen\" enable=\"True\">
			<Options>0</Options>
		</Step>";

        let expected_output = Some("Ergebnismenge suchen".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_with_criteria() {
        let xml_input = "
        <Step id=\"28\" name=\"Ergebnismenge suchen\" enable=\"True\">
            <Options>33587200</Options>
            <ParameterValues membercount=\"1\">
                <Parameter type=\"FindRequest\">
                    <FindRequestSet membercount=\"2\">
                        <FindRequest membercount=\"2\" action=\"find\">
                            <find criteria=\"ScriptStep\">
                                <FieldReference id=\"15\" name=\"Selector\">
                                    <TableOccurrenceReference id=\"1065113\" name=\"_Syntax\"></TableOccurrenceReference>
                                </FieldReference>
                            </find>
                            <find criteria=\"1\">
                                <FieldReference id=\"21\" name=\"_kIsWildcard\">
                                    <TableOccurrenceReference id=\"1065113\" name=\"_Syntax\"></TableOccurrenceReference>
                                </FieldReference>
                            </find>
                        </FindRequest>
                        <FindRequest membercount=\"1\" action=\"omit\">
                            <find criteria=\"1&#13;2\">
                                <FieldReference id=\"12\" name=\"__ID\">
                                    <TableOccurrenceReference id=\"1065113\" name=\"_Syntax\"></TableOccurrenceReference>
                                </FieldReference>
                            </find>
                        </FindRequest>
                    </FindRequestSet>
                </Parameter>
            </ParameterValues>
        </Step>
        ";

        let expected_output = Some("Ergebnismenge suchen [ ⚠️ RESTORE ⚠️ ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }
}
