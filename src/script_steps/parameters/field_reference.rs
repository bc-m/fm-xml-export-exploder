use crate::script_steps::parameters::calculation::Calculation;
use crate::utils::attributes::{get_attribute, get_attributes};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

#[derive(Debug, Default)]
pub struct FieldReference {
    pub field_reference: String,
}

impl FieldReference {
    pub fn from_xml(reader: &mut Reader<&[u8]>, e: &BytesStart) -> Result<Self, String> {
        let mut depth = 1;
        let mut item = FieldReference {
            field_reference: get_attribute(e, "name").unwrap_or_default(),
        };

        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if e.name().as_ref() == b"TableOccurrenceReference" {
                        for attr in get_attributes(&e).unwrap() {
                            if attr.0 == "name" {
                                match e.name().as_ref() {
                                    b"TableOccurrenceReference" => {
                                        item.field_reference = format!(
                                            "{}::{}",
                                            attr.1,
                                            match item.field_reference.is_empty() {
                                                true => "🚨🚨🚨 <BROKEN REFERENCE> 🚨🚨🚨",
                                                false => item.field_reference.as_str(),
                                            }
                                        );
                                    }
                                    b"Calculation" => {
                                        item.field_reference =
                                            Calculation::from_xml(reader, &e).unwrap();
                                    }

                                    _ => {}
                                }
                            }
                        }
                    }
                }
                Ok(Event::End(_)) => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            buf.clear();
        }

        if item.field_reference.is_empty() {
            item.field_reference = "🚨🚨🚨 BROKEN REFERENCE 🚨🚨🚨".to_string();
        }

        Ok(item)
    }

    pub fn display(&self) -> Option<String> {
        Some(self.field_reference.to_string())
    }
}

#[derive(Debug, Default)]
pub struct FieldReferenceParameter {
    pub field_reference: String,
}

impl FieldReferenceParameter {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _: &BytesStart) -> Result<Self, String> {
        let mut depth = 1;
        let mut item = FieldReferenceParameter::default();

        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if e.name().as_ref() == b"FieldReference" {
                        item.field_reference = FieldReference::from_xml(reader, &e)
                            .unwrap()
                            .display()
                            .unwrap();
                        depth -= 1;
                    }
                }
                Ok(Event::End(_)) => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            buf.clear();
        }

        if item.field_reference.is_empty() {
            item.field_reference = "🚨🚨🚨 NO REFERENCE 🚨🚨🚨".to_string();
        }

        Ok(item)
    }

    pub fn display(&self) -> Option<String> {
        Some(self.field_reference.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::script_steps::parameters::field_reference::{
        FieldReference, FieldReferenceParameter,
    };
    use quick_xml::events::Event;
    use quick_xml::Reader;

    #[test]
    fn test_field_reference() {
        let xml_input = "<FieldReference id=\"4\" name=\"Bar\">
						<repetition value=\"1\"></repetition>
						<TableOccurrenceReference id=\"1065090\" name=\"Foo\"></TableOccurrenceReference>
					</FieldReference>";

        let mut reader = Reader::from_str(xml_input);
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "Foo::Bar".to_string();
        assert_eq!(
            FieldReference::from_xml(&mut reader, &element)
                .unwrap()
                .display()
                .unwrap(),
            expected_output
        );
    }

    #[test]
    fn test_field_referenc_parameter() {
        let xml_input = "<Parameter type=\"FieldReference\">
            <FieldReference id=\"2\" name=\"Bar\">
                <repetition value=\"1\"></repetition>
                <TableOccurrenceReference id=\"1065123\" name=\"Foo\"></TableOccurrenceReference>
            </FieldReference>
        </Parameter>";

        let mut reader = Reader::from_str(xml_input);
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "Foo::Bar".to_string();
        assert_eq!(
            FieldReferenceParameter::from_xml(&mut reader, &element)
                .unwrap()
                .display()
                .unwrap(),
            expected_output
        );
    }
}
