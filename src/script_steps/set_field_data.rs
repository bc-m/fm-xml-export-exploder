use quick_xml::events::Event;
use quick_xml::Reader;

use crate::script_steps::parameters::calculation::Calculation;
use crate::script_steps::parameters::field_reference::FieldReference;
use crate::utils::attributes::get_attribute;

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut field_reference = String::new();
    let mut calculation = String::new();

    let mut reader = Reader::from_str(step);
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => name = get_attribute(&e, "name").unwrap().to_string(),
                b"FieldReference" => {
                    field_reference = FieldReference::from_xml(&mut reader, &e)
                        .unwrap()
                        .display()
                        .unwrap()
                }
                b"Calculation" => {
                    calculation = Calculation::from_xml(&mut reader, &e).unwrap();
                }
                _ => {}
            },
            _ => {}
        }
        buf.clear()
    }

    if name.is_empty() {
        None
    } else {
        Some(format!(
            "{} [ {} ; {} ]",
            name, field_reference, calculation
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let xml = r#"
            <Step id="76" name="Feldwert setzen" enable="True">
                <Options>16385</Options>
                <ParameterValues membercount="2">
                    <Parameter type="FieldReference">
                        <FieldReference id="4" name="FieldFoo">
                            <repetition value="1"></repetition>
                            <TableOccurrenceReference id="1065090" name="TableFoo"></TableOccurrenceReference>
                        </FieldReference>
                    </Parameter>
                    <Parameter type="Calculation">
                        <Calculation datatype="1" position="0">
                            <Calculation>
                                <Text><![CDATA[TableBar::FieldBar]]></Text>
                                <ChunkList hash="83065BB32DCAC16FD7B0ABECE2D7A367">
                                    <Chunk type="FieldRef">
                                        <FieldReference id="4" name="FieldBar" repetition="1">
                                            <TableOccurrenceReference id="1065089" name="TableBar"></TableOccurrenceReference>
                                        </FieldReference>
                                    </Chunk>
                                </ChunkList>
                            </Calculation>
                        </Calculation>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output =
            Some("Feldwert setzen [ TableFoo::FieldFoo ; TableBar::FieldBar ]".to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }
}
