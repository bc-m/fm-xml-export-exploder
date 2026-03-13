use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::utils::attributes::get_attribute;

#[derive(Debug, Default)]
pub struct VariableReference {
    pub name: Option<String>,
    pub repetition: Option<i32>,
}

impl VariableReference {
    pub fn from_xml(reader: &mut Reader<&[u8]>, e: &BytesStart) -> VariableReference {
        let mut depth = 1;
        let mut item = VariableReference {
            name: get_attribute(e, "value"),
            ..Default::default()
        };

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if e.name().as_ref() == b"repetition"
                        && let Some(repetition) = get_attribute(&e, "value")
                        && let Ok(repetition) = repetition.parse::<i32>()
                    {
                        item.repetition = Some(repetition)
                    };
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

    pub fn display(self) -> Option<String> {
        self.name.map(|name| match self.repetition {
            Some(rep) if rep != 1 => format!("{name}[{rep}]"),
            _ => name,
        })
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

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
                .display()
                .unwrap(),
            expected_output
        );
    }
}
