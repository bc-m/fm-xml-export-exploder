use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::script_steps::parameters::boolean::Boolean;
use crate::utils::xml_utils::{local_name_to_string, text_to_string};

#[derive(Debug, Default)]
pub struct BooleanContainer {
    pub name: String,
    pub value: Option<bool>,
}

impl BooleanContainer {
    pub fn from_xml(reader: &mut Reader<&[u8]>, e: &BytesStart) -> BooleanContainer {
        let mut depth = 1;
        let mut item = BooleanContainer {
            name: local_name_to_string(e.name().as_ref()),
            ..Default::default()
        };

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(_)) => depth += 1,
                Ok(Event::Text(e)) => match text_to_string(&e).as_str() {
                    "True" => item.value = Some(true),
                    "False" => item.value = Some(false),
                    _ => {}
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
        // Hide DimParentWindow entirely
        if self.name == "DimParentWindow" {
            return None;
        }

        let value = self.value?;

        // Hide other window options when they are ON (their default)
        if value
            && matches!(
                self.name.as_str(),
                "Close" | "Minimize" | "Maximize" | "Resize" | "MenuBar" | "Toolbar"
            )
        {
            return None;
        }

        let label = self.name.replace("MenuBar", "Menu");
        Some(format!("{label}: {}", Boolean::on_off(value)))
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    use crate::script_steps::parameters::boolean_container::BooleanContainer;

    #[test]
    fn test() {
        let xml = r#"<Close>False</Close>"#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = "Close: OFF".to_string();
        assert_eq!(
            BooleanContainer::from_xml(&mut reader, &element)
                .display()
                .unwrap_or_default(),
            expected_output
        );
    }
}
