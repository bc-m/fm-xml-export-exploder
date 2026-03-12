use quick_xml::Reader;
use quick_xml::events::Event;

use crate::script_steps::parameters::calculation::Calculation;
use crate::utils::attributes::get_attribute;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut option_name = String::new();
    let mut state = false;
    let mut calculation = String::new();

    let mut reader = Reader::from_str(step);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => {
                    name = get_attribute(&e, "name").unwrap_or_default();
                    continue;
                }
                b"Boolean" => {
                    option_name = get_attribute(&e, "type").unwrap_or_default();
                    state = get_attribute(&e, "value").is_some_and(|v| v == "True");
                    continue;
                }
                b"Calculation" => {
                    calculation = Calculation::from_xml(&mut reader, &e)
                        .unwrap()
                        .display()
                        .unwrap()
                }
                _ => {}
            },
            _ => {}
        }
        buf.clear()
    }

    if option_name.is_empty() && calculation.is_empty() {
        return Some(format!("{name} []"));
    }

    let on_off = if state { "ON" } else { "OFF" };

    if !calculation.is_empty() {
        Some(format!(
            "{name} [ {option_name}: {on_off} ; {calculation} ]"
        ))
    } else {
        Some(format!("{name} [ {option_name}: {on_off} ]"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let xml = r#"
            <Step id="26" name="Mehrere ausschließen" enable="True">
                <ParameterValues membercount="1">
                    <Parameter type="Boolean">
                        <Boolean type="Mit Dialog" id="128" value="True"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Mehrere ausschließen [ Mit Dialog: ON ]".to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn test_with_calculation() {
        let xml = r#"
            <Step id="26" name="Mehrere ausschließen" enable="True">
                <Options>16512</Options>
                <ParameterValues membercount="2">
                    <Parameter type="Boolean">
                        <Boolean type="Mit Dialog" id="128" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Calculation">
                        <Calculation datatype="1" position="0">
                            <Calculation>
                                <Text><![CDATA[123]]></Text>
                                <ChunkList hash="90213DC459B04C5A47C044B9460AEF7B">
                                    <Chunk type="NoRef">123</Chunk>
                                </ChunkList>
                            </Calculation>
                        </Calculation>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Mehrere ausschließen [ Mit Dialog: OFF ; 123 ]".to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }
}
