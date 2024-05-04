use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::script_steps::parameters::field_reference::FieldReference;
use crate::script_steps::parameters::variable_reference::VariableReference;

#[derive(Debug, Default)]
pub struct Target {
    pub target: Option<String>,
}

impl Target {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _e: &BytesStart) -> Result<Target, String> {
        let mut depth = 1;
        let mut item = Target { target: None };

        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"Variable" => {
                        item.target = VariableReference::from_xml(reader, &e).unwrap().display();
                    }
                    b"FieldReference" => {
                        item.target = FieldReference::from_xml(reader, &e).unwrap().display();
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

        Ok(item)
    }

    pub fn display(&self) -> Option<String> {
        self.target.clone()
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::events::Event;
    use quick_xml::Reader;

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
            Target::from_xml(&mut reader, &element)
                .unwrap()
                .display()
                .unwrap(),
            expected_output
        );
    }
}
