use crate::calculations::calculation::Calculation;
use crate::utils::attributes::get_attribute;
use quick_xml::events::Event;
use quick_xml::Reader;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut option_name = String::new();
    let mut state = false;
    let mut calculation = String::new();

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
                b"Calculation" => calculation = Calculation::from_xml(&mut reader, &e).unwrap(),
                _ => {}
            },
            _ => {}
        }
        buf.clear()
    }

    if option_name.is_empty() && calculation.is_empty() {
        Some(format!("{} []", name))
    } else if !calculation.is_empty() {
        Some(format!(
            "{} [ {}: {} ; {} ]",
            name,
            option_name,
            match state {
                true => "ON",
                false => "OFF",
            },
            calculation
        ))
    } else {
        Some(format!(
            "{} [ {}: {} ]",
            name,
            option_name,
            match state {
                true => "ON",
                false => "OFF",
            }
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let xml_input = "\
        <Step id=\"26\" name=\"Mehrere ausschließen\" enable=\"True\">
            <ParameterValues membercount=\"1\">
                <Parameter type=\"Boolean\">
                    <Boolean type=\"Mit Dialog\" id=\"128\" value=\"True\"></Boolean>
                </Parameter>
            </ParameterValues>
        </Step>";

        let expected_output = Some("Mehrere ausschließen [ Mit Dialog: ON ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }

    #[test]
    fn test_with_calculation() {
        let xml_input = "
        <Step id=\"26\" name=\"Mehrere ausschließen\" enable=\"True\">
            <Options>16512</Options>
            <ParameterValues membercount=\"2\">
                <Parameter type=\"Boolean\">
                    <Boolean type=\"Mit Dialog\" id=\"128\" value=\"False\"></Boolean>
                </Parameter>
                <Parameter type=\"Calculation\">
                    <Calculation datatype=\"1\" position=\"0\">
                        <Calculation>
                            <Text><![CDATA[123]]></Text>
                            <ChunkList hash=\"90213DC459B04C5A47C044B9460AEF7B\">
                                <Chunk type=\"NoRef\">123</Chunk>
                            </ChunkList>
                        </Calculation>
                    </Calculation>
                </Parameter>
            </ParameterValues>
        </Step>";

        let expected_output = Some("Mehrere ausschließen [ Mit Dialog: OFF ; 123 ]".to_string());
        assert_eq!(sanitize(xml_input.trim()), expected_output);
    }
}
