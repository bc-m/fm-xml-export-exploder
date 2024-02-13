use crate::utils::attributes::get_attribute;
use quick_xml::events::Event;
use quick_xml::Reader;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut option_name = String::new();
    let mut state = false;

    let mut reader = Reader::from_str(step);
    reader.trim_text(true);
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
                b"Boolean" => {
                    if let Some(name) = get_attribute(&e, "type") {
                        option_name = name.to_string();
                    }

                    match get_attribute(&e, "value").unwrap().as_str() {
                        "True" => {
                            state = true;
                        }
                        "False" => {
                            state = false;
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

    let state_string = match state {
        true => "ON",
        false => "OFF",
    };

    if option_name.is_empty() {
        Some(format!("{} [ {} ]", name, state_string))
    } else {
        Some(format!("{} [ {}: {} ]", name, option_name, state_string))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_enabled() {
        let xml_input = "
		<Step id=\"86\" name=\"Fehleraufzeichnung setzen\" enable=\"True\">
			<Options>196608</Options>
			<ParameterValues membercount=\"1\">
				<Parameter type=\"Boolean\">
					<Boolean id=\"131072\" value=\"True\"></Boolean>
				</Parameter>
			</ParameterValues>
		</Step>
        ";

        let expected_output = Some("Fehleraufzeichnung setzen [ ON ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_sanitize_disabled() {
        let xml_input = "
		<Step id=\"86\" name=\"Fehleraufzeichnung setzen\" enable=\"True\">
			<Options>196608</Options>
			<ParameterValues membercount=\"1\">
				<Parameter type=\"Boolean\">
					<Boolean id=\"131072\" value=\"False\"></Boolean>
				</Parameter>
			</ParameterValues>
		</Step>
        ";

        let expected_output = Some("Fehleraufzeichnung setzen [ OFF ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_sanitize_enter_find_mode() {
        let xml_input = "
        <Step id=\"22\" name=\"Suchenmodus aktivieren\" enable=\"True\">
			<SourceUUID>3C5EBF9C-BAE0-460F-92AF-3FB73649951B</SourceUUID>
			<ParameterValues membercount=\"1\">
				<Parameter type=\"Boolean\">
					<Boolean type=\"Pause\" id=\"16777216\" value=\"False\"></Boolean>
				</Parameter>
			</ParameterValues>
		</Step>
        ";

        let expected_output = Some("Suchenmodus aktivieren [ Pause: OFF ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }
}
