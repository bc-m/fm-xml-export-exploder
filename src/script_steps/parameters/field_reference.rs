use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::utils::attributes::get_attribute;

#[derive(Debug, Default)]
pub struct FieldReference {
    pub table_reference: Option<String>,
    pub field_reference: Option<String>,
    pub repetition: Option<i32>,
}

impl FieldReference {
    pub fn from_xml(reader: &mut Reader<&[u8]>, e: &BytesStart) -> FieldReference {
        let mut depth = 1;
        let mut item = FieldReference {
            field_reference: get_attribute(e, "name"),
            ..Default::default()
        };

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    match e.name().as_ref() {
                        b"FieldReference" => {
                            item.field_reference = get_attribute(&e, "name");
                        }
                        b"TableOccurrenceReference" => {
                            item.table_reference = get_attribute(&e, "name");
                        }
                        b"repetition" => {
                            item.repetition =
                                get_attribute(&e, "value").and_then(|v| v.parse().ok());
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

        item
    }

    pub fn display(&self) -> String {
        let Some(table_reference) = &self.table_reference else {
            return "🚨🚨🚨 BROKEN REFERENCE 🚨🚨🚨".to_string();
        };

        let field_reference = match &self.field_reference {
            Some(reference) if !reference.is_empty() => reference.as_str(),
            _ => "🚨🚨🚨 <BROKEN REFERENCE> 🚨🚨🚨",
        };

        match self.repetition {
            Some(rep) if rep != 1 => format!("{table_reference}::{field_reference}[{rep}]"),
            _ => format!("{table_reference}::{field_reference}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    use crate::script_steps::parameters::field_reference::FieldReference;

    #[test]
    fn test_field_reference() {
        let xml = r#"
            <FieldReference id="4" name="Bar">
                <repetition value="1"></repetition>
                <TableOccurrenceReference id="1065090" name="Foo"></TableOccurrenceReference>
            </FieldReference>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "Foo::Bar";
        assert_eq!(
            FieldReference::from_xml(&mut reader, &element).display(),
            expected_output
        );
    }

    #[test]
    fn test_field_reference_with_repetition() {
        let xml = r#"
            <FieldReference id="4" name="Bar">
                <repetition value="1337"></repetition>
                <TableOccurrenceReference id="1065090" name="Foo"></TableOccurrenceReference>
            </FieldReference>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "Foo::Bar[1337]";
        assert_eq!(
            FieldReference::from_xml(&mut reader, &element).display(),
            expected_output
        );
    }
}
