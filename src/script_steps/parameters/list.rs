use crate::script_steps::constants::{id_to_script_step, ScriptStep};
use quick_xml::escape::unescape;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::utils::attributes::get_attribute;

#[derive(Debug, Default)]
pub struct List {
    pub name: Option<String>,
}

impl List {
    pub fn from_xml(
        reader: &mut Reader<&[u8]>,
        _: &BytesStart,
        step_id: &u32,
    ) -> Result<List, String> {
        let mut depth = 1;
        let mut item = List { name: None };

        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if let b"List" = e.name().as_ref() {
                        if let Some(name) = get_attribute(&e, "name") {
                            if let Ok(name) = unescape(name.as_str()) {
                                item.name = match id_to_script_step(step_id) {
                                    ScriptStep::LoopStart => Some(format!("Flush: {}", name)),
                                    _ => Some(name.to_string()),
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

        Ok(item)
    }

    pub fn display(&self) -> Option<String> {
        self.name.clone()
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    use crate::script_steps::parameters::list::List;

    #[test]
    fn test() {
        let xml = r#"
            <Parameter type="List">
                <List id="136" name="_Home" UUID="C012F95A-8FC4-41A9-B354-2F21FE641437"></List>
            </Parameter>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "_Home".to_string();
        let script_id: u32 = 136;
        assert_eq!(
            List::from_xml(&mut reader, &element, &script_id)
                .unwrap()
                .display()
                .unwrap(),
            expected_output
        );
    }
}
