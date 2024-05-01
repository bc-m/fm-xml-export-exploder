use crate::script_steps::parameters::calculation::Calculation;
use crate::utils::attributes::{get_attribute, get_attributes};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

#[derive(Debug, Default)]
pub struct FieldReference {
    pub table_reference: Option<String>,
    pub field_reference: Option<String>,
    pub repetition: Option<i32>,
}

impl FieldReference {
    pub fn from_xml(reader: &mut Reader<&[u8]>, e: &BytesStart) -> Result<FieldReference, String> {
        let mut depth = 1;
        let mut item = FieldReference {
            table_reference: None,
            field_reference: get_attribute(e, "name"),
            repetition: None,
        };

        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    match e.name().as_ref() {
                        b"TableOccurrenceReference" => {
                            for attr in get_attributes(&e).unwrap() {
                                if attr.0 == "name" {
                                    match e.name().as_ref() {
                                        b"TableOccurrenceReference" => {
                                            item.table_reference = Option::from(attr.1);
                                        }
                                        b"Calculation" => {
                                            item.field_reference = Option::from(
                                                Calculation::from_xml(reader, &e).unwrap(),
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        b"repetition" => {
                            match get_attribute(&e, "value") {
                                None => {}
                                Some(repetition) => {
                                    if let Ok(repetition) = repetition.parse::<i32>() {
                                        item.repetition = Some(repetition)
                                    }
                                }
                            };
                        }
                        _ => {}
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

        Ok(item)
    }

    pub fn display(&self) -> Option<String> {
        let table_reference: String = match &self.table_reference {
            None => return Some("🚨🚨🚨 BROKEN REFERENCE 🚨🚨🚨".to_string()),
            Some(reference) => reference.clone(),
        };

        let field_reference = match &self.field_reference {
            None => "🚨🚨🚨 <BROKEN REFERENCE> 🚨🚨🚨".to_string(),
            Some(reference) => {
                if reference.is_empty() {
                    "🚨🚨🚨 <BROKEN REFERENCE> 🚨🚨🚨".to_string()
                } else {
                    reference.clone()
                }
            }
        };

        let repetition = self.repetition.unwrap_or(1);
        if repetition != 1 {
            Some(format!(
                "{}::{}[{}]",
                table_reference.as_str(),
                field_reference.as_str(),
                repetition
            ))
        } else {
            Some(format!(
                "{}::{}",
                table_reference.as_str(),
                field_reference.as_str()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::script_steps::parameters::field_reference::FieldReference;
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
    fn test_field_reference_with_repetition() {
        let xml_input = "<FieldReference id=\"4\" name=\"Bar\">
						<repetition value=\"1337\"></repetition>
						<TableOccurrenceReference id=\"1065090\" name=\"Foo\"></TableOccurrenceReference>
					</FieldReference>";

        let mut reader = Reader::from_str(xml_input);
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "Foo::Bar[1337]".to_string();
        assert_eq!(
            FieldReference::from_xml(&mut reader, &element)
                .unwrap()
                .display()
                .unwrap(),
            expected_output
        );
    }
}
