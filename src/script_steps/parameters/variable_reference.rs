use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::utils::attributes::get_attribute;

#[derive(Debug, Default)]
pub struct VariableReference {
    pub name: Option<String>,
    pub repetition: Option<i32>,
}

impl VariableReference {
    pub fn from_xml(
        reader: &mut Reader<&[u8]>,
        e: &BytesStart,
    ) -> Result<VariableReference, String> {
        let mut depth = 1;
        let mut item = VariableReference {
            name: get_attribute(e, "value"),
            repetition: None,
        };

        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if e.name().as_ref() == b"repetition" {
                        if let Some(repetition) = get_attribute(&e, "value") {
                            if let Ok(repetition) = repetition.parse::<i32>() {
                                item.repetition = Some(repetition)
                            }
                        };
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
        if let Some(name) = &self.name {
            let repetition = self.repetition.unwrap_or(1);
            if repetition != 1 {
                Some(format!("{}[{}]", name, repetition))
            } else {
                Some(name.clone())
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    use crate::script_steps::parameters::variable_reference::VariableReference;

    #[test]
    fn test() {
        let xml = r#"
            <Variable value="$foo">
                <repetition value="1"></repetition>
            </Variable>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "$foo".to_string();
        assert_eq!(
            VariableReference::from_xml(&mut reader, &element)
                .unwrap()
                .display()
                .unwrap(),
            expected_output
        );
    }

    #[test]
    fn test_with_repetition() {
        let xml = r#"
            <Variable value="$foo">
                <repetition value="1337"></repetition>
            </Variable>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "$foo[1337]".to_string();
        assert_eq!(
            VariableReference::from_xml(&mut reader, &element)
                .unwrap()
                .display()
                .unwrap(),
            expected_output
        );
    }
}
