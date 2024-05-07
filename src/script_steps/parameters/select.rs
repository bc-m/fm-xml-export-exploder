use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::script_steps::parameters::calculation::Calculation;

#[derive(Debug, Default)]
pub struct Select {
    pub text: Option<String>,
}

impl Select {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _: &BytesStart) -> Result<Select, String> {
        let mut depth = 1;
        let mut item = Select { text: None };

        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if e.name().as_ref() == b"Calculation" {
                        if let Ok(param_value) = Calculation::from_xml(reader, &e) {
                            if let Some(display) = param_value.display() {
                                item.text = Some(format!("Name: {}", display));
                            }
                        }
                        depth -= 1;
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
        self.text.clone()
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    use crate::script_steps::parameters::select::Select;

    #[test]
    fn test() {
        let xml = r#"<Select kind="0" type="current"></Select>"#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        assert_eq!(
            Select::from_xml(&mut reader, &element).unwrap().display(),
            None
        );
    }

    #[test]
    fn test_calc_name() {
        let xml = r#"
            <Select kind="1" type="Calculated">
                <Name current="True">
                    <Calculation datatype="1" position="0">
                        <Calculation>
                            <Text><![CDATA[$FensterName]]></Text>
                            <ChunkList hash="F664CE455D905B4C61115DE802697AE3">
                                <Chunk type="VariableReference">$FensterName</Chunk>
                            </ChunkList>
                        </Calculation>
                    </Calculation>
                </Name>
            </Select>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        assert_eq!(
            Select::from_xml(&mut reader, &element).unwrap().display(),
            Some("Name: $FensterName".to_string())
        );
    }
}
