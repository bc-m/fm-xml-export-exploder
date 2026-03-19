use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::utils::attributes::parse_unescaped_attribute;

#[derive(Debug, Default)]
pub struct PrivilegeSetReference {
    pub name: Option<String>,
}

impl PrivilegeSetReference {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _: &BytesStart) -> PrivilegeSetReference {
        let mut depth = 1;
        let mut item = PrivilegeSetReference::default();

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if e.name().as_ref() == b"PrivilegeSetReference" {
                        item.name = parse_unescaped_attribute(&e, "name");
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

    pub fn display(self) -> Option<String> {
        self.name
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    use crate::script_steps::parameters::privilege_set_reference::PrivilegeSetReference;

    #[test]
    fn test_known() {
        let xml = r#"
            <Parameter type="PrivilegeSetReference">
                <PrivilegeSetReference id="2" name="[Data Entry Only]"></PrivilegeSetReference>
            </Parameter>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "[Data Entry Only]".to_string();
        assert_eq!(
            PrivilegeSetReference::from_xml(&mut reader, &element)
                .display()
                .unwrap(),
            expected_output
        );
    }

    #[test]
    fn test_unknown() {
        let xml = r#"
            <Parameter type="PrivilegeSetReference">
                <PrivilegeSetReference id="0" name="&lt;unknown&gt;"></PrivilegeSetReference>
            </Parameter>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "<unknown>".to_string();
        assert_eq!(
            PrivilegeSetReference::from_xml(&mut reader, &element)
                .display()
                .unwrap(),
            expected_output
        );
    }
}
