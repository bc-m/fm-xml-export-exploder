use quick_xml::events::Event;
use quick_xml::Reader;

use crate::script_steps::parameters::calculation::Calculation;
use crate::utils::attributes::get_attribute;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut calculation = String::new();
    let mut repetition = String::new();

    let mut reader = Reader::from_str(step);
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"Step" => {
                    name = get_attribute(e, "name").unwrap().to_string();
                }
                b"Name" => {
                    calculation = Calculation::from_xml(&mut reader, e)
                        .unwrap()
                        .display()
                        .unwrap_or_default()
                }
                b"repetition" => {
                    repetition = Calculation::from_xml(&mut reader, e)
                        .unwrap()
                        .display()
                        .unwrap_or_default()
                }
                _ => {}
            },
            _ => {}
        }
        buf.clear()
    }

    if name.is_empty() {
        None
    } else if !calculation.is_empty() && !repetition.is_empty() {
        Some(format!(
            "{name} [ {calculation} ; Repetition: {repetition} ]"
        ))
    } else if !calculation.is_empty() {
        Some(format!("{name} [ {calculation} ]"))
    } else if !repetition.is_empty() {
        Some(format!("{name} [ ; Repetition: {repetition} ]"))
    } else {
        Some(format!("{name} []"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let xml = r#"
            <Step id="145" name="Gehe zu Objekt" enable="True">
                <SourceUUID>CAEB554C-8BA5-4074-BCEF-EAF86C7744A3</SourceUUID>
                <Options>16384</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Object">
                        <Name>
                            <Calculation datatype="1" position="0">
                                <Calculation>
                                    <Text><![CDATA["Foo Bar"]]></Text>
                                    <ChunkList hash="DE83BCE7DDEC586C5CBA59FCA8CB05CC">
                                        <Chunk type="NoRef">&quot;Foo Bar&quot;</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </Name>
                        <repetition></repetition>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(r#"Gehe zu Objekt [ "Foo Bar" ]"#.to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn test_repetition() {
        let xml = r#"
            <Step id="145" name="Gehe zu Objekt" enable="True">
                <SourceUUID>CAEB554C-8BA5-4074-BCEF-EAF86C7744A3</SourceUUID>
                <Options>16384</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Object">
                        <Name>
                            <Calculation datatype="1" position="0">
                                <Calculation>
                                    <Text><![CDATA["Foo Bar"]]></Text>
                                    <ChunkList hash="DE83BCE7DDEC586C5CBA59FCA8CB05CC">
                                        <Chunk type="NoRef">&quot;Foo Bar&quot;</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </Name>
                        <repetition>
                            <Calculation datatype="1" position="1">
                                <Calculation>
                                    <Text><![CDATA[$Rep]]></Text>
                                    <ChunkList hash="5EFB6D6F00E58E6BB5E399274B87A065">
                                        <Chunk type="VariableReference">$Rep</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </repetition>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output =
            Some(r#"Gehe zu Objekt [ "Foo Bar" ; Repetition: $Rep ]"#.to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }
}
