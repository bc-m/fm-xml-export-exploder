use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::script_steps::parameters::field_reference::FieldReference;
use crate::script_steps::parameters::variable_reference::VariableReference;

#[derive(Debug, Default)]
pub struct Target {
    pub target: Option<String>,
}

impl Target {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _: &BytesStart) -> Target {
        let mut depth = 1;
        let mut item = Target::default();

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"Variable" => {
                        item.target = VariableReference::from_xml(reader, &e).display();
                    }
                    b"FieldReference" => {
                        item.target = Some(FieldReference::from_xml(reader, &e).display());
                    }
                    _ => {
                        depth += 1;
                    }
                },
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
        self.target
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    use crate::script_steps::parameters::target::Target;

    #[test]
    fn test() {
        let xml = r#"
            <Parameter type="Target">
                <Variable value="$foo">
                    <repetition value="1"></repetition>
                </Variable>
            </Parameter>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "$foo".to_string();
        assert_eq!(
            Target::from_xml(&mut reader, &element).display().unwrap(),
            expected_output
        );
    }
}
