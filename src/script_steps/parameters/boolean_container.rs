use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::utils::xml_utils::{local_name_to_string, text_to_string};

#[derive(Debug, Default)]
pub struct BooleanContainer {
    pub name: String,
    pub value: Option<bool>,
}

impl BooleanContainer {
    pub fn from_xml(
        reader: &mut Reader<&[u8]>,
        e: &BytesStart,
    ) -> Result<BooleanContainer, String> {
        let mut depth = 1;
        let mut item = BooleanContainer {
            name: local_name_to_string(e.name().as_ref()),
            value: None,
        };

        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(_)) => {
                    depth += 1;
                }
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

        Ok(item)
    }

    pub fn bool_to_string(bool: bool) -> String {
        match bool {
            true => "ON".to_string(),
            false => "OFF".to_string(),
        }
    }

    pub fn display(&self) -> Option<String> {
        if self.name.as_str() == "DimParentWindow" {
            return None;
        }

        if matches!(
            self.name.as_str(),
            "Close" | "Minimize" | "Maximize" | "Resize" | "MenuBar" | "Toolbar"
        ) && self.value.unwrap()
        {
            return None;
        }

        self.value.map(|bool_value| {
            let formatted_string = format!(
                "{}: {}",
                self.name.replace("MenuBar", "Menu"),
                Self::bool_to_string(bool_value)
            );
            formatted_string
        })
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::events::Event;
    use quick_xml::Reader;

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
                .unwrap()
                .display()
                .unwrap_or_default(),
            expected_output
        );
    }
}
