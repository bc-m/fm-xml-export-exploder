use crate::calculations::calculation::Calculation;
use crate::utils::attributes::get_attribute;
use crate::utils::xml_utils::text_to_string;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

#[derive(Debug, Default)]
pub struct LayoutReferenceContainer {
    pub reference_type: String,
    pub layout_reference: String,
}

impl LayoutReferenceContainer {
    pub fn parse_label(reader: &mut Reader<&[u8]>, _: &BytesStart) -> Result<String, String> {
        let mut label = String::new();
        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Text(e)) => {
                    label = text_to_string(&e);
                }
                Ok(Event::End(_)) => break,
                _ => {}
            }
            buf.clear();
        }

        Ok(label)
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>, e: &BytesStart) -> Result<Self, String> {
        let mut depth = 1;
        let mut item = LayoutReferenceContainer {
            reference_type: get_attribute(e, "value").unwrap().to_string(),
            ..Default::default()
        };

        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if item.reference_type.as_str() == "0" {
                        continue;
                    };
                    match e.name().as_ref() {
                        b"LayoutReference" => {
                            item.layout_reference = get_attribute(&e, "name").unwrap().to_string();
                            break;
                        }
                        b"Label" => {
                            item.layout_reference =
                                LayoutReferenceContainer::parse_label(reader, &e).unwrap();
                            depth -= 1;
                        }
                        b"Calculation" => {
                            item.layout_reference =
                                Calculation::from_xml(reader, &e).unwrap_or_default();
                            depth -= 1;
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

        Ok(item)
    }

    pub fn display(&self) -> Option<String> {
        if self.reference_type == "0" {
            Some("".to_string())
        } else if self.reference_type == "1" {
            Some(format!("Layout: <{}>", self.layout_reference))
        } else if self.reference_type == "3" {
            Some(format!("{}: {}", "Layoutname", self.layout_reference))
        } else if self.reference_type == "4" {
            Some(format!("{}: {}", "Layoutnr.", self.layout_reference))
        } else if self.reference_type == "5" {
            Some(format!("{}: \"{}\"", "Layout", self.layout_reference))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let xml_input = "<LayoutReferenceContainer value=\"1\"><Label>Originallayout</Label></LayoutReferenceContainer>";

        let mut reader = Reader::from_str(xml_input);
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = Some("Layout: <Originallayout>".to_string());
        assert_eq!(
            LayoutReferenceContainer::from_xml(&mut reader, &element)
                .unwrap()
                .display(),
            expected_output
        );
    }

    #[test]
    fn test_layout() {
        let xml_input = "\
        <LayoutReferenceContainer value=\"5\">\
            <LayoutReference id=\"2\" name=\"Aufgabenliste\" UUID=\"EBFC887D-CF13-4ABE-BB71-78EDC2B30FEE\"></LayoutReference>
        </LayoutReferenceContainer>
        ";

        let mut reader = Reader::from_str(xml_input);
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = Some("Layout: \"Aufgabenliste\"".to_string());
        assert_eq!(
            LayoutReferenceContainer::from_xml(&mut reader, &element)
                .unwrap()
                .display(),
            expected_output
        );
    }

    #[test]
    fn test_by_name() {
        let xml_input = "
        <LayoutReferenceContainer value=\"3\">
            <Calculation datatype=\"1\" position=\"5\">
                <Calculation>
                    <Text><![CDATA[\"LAYOUT_NAME_CALCULATION\"]]></Text>
                    <ChunkList hash=\"56E1B7D90047D6C6C4421314C701E408\">
                        <Chunk type=\"NoRef\">&quot;LAYOUT_NAME_CALCULATION&quot;</Chunk>
                    </ChunkList>
                </Calculation>
            </Calculation>
        </LayoutReferenceContainer>
        ";

        let mut reader = Reader::from_str(xml_input);
        reader.trim_text(true);
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = Some("Layoutname: \"LAYOUT_NAME_CALCULATION\"".to_string());
        assert_eq!(
            LayoutReferenceContainer::from_xml(&mut reader, &element)
                .unwrap()
                .display(),
            expected_output
        );
    }

    #[test]
    fn test_by_number() {
        let xml_input = "
        <LayoutReferenceContainer value=\"4\">
            <Calculation datatype=\"1\" position=\"5\">
                <Calculation>
                    <Text><![CDATA[\"LAYOUT_NUMBER_CALCULATION\"]]></Text>
                    <ChunkList hash=\"3C7B0A49FD772FDF6F8218753BDDDE7D\">
                        <Chunk type=\"NoRef\">&quot;LAYOUT_NUMBER_CALCULATION&quot;</Chunk>
                    </ChunkList>
                </Calculation>
            </Calculation>
        </LayoutReferenceContainer>
        ";

        let mut reader = Reader::from_str(xml_input);
        reader.trim_text(true);

        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = Some("Layoutnr.: \"LAYOUT_NUMBER_CALCULATION\"".to_string());
        assert_eq!(
            LayoutReferenceContainer::from_xml(&mut reader, &element)
                .unwrap()
                .display(),
            expected_output
        );
    }
}
