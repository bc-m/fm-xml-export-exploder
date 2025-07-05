use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::script_steps::parameters::calculation::Calculation;
use crate::utils::attributes::get_attribute;
use crate::utils::xml_utils::text_to_string;

#[derive(Debug, Default)]
pub struct LayoutReferenceContainer {
    pub reference_type: String,
    pub layout_reference: Option<String>,
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
            reference_type: get_attribute(e, "value")
                .unwrap_or("".to_string())
                .to_string(),
            ..Default::default()
        };

        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    match e.name().as_ref() {
                        b"LayoutReferenceContainer" => {
                            item.reference_type = get_attribute(&e, "value")
                                .unwrap_or("".to_string())
                                .to_string();
                        }
                        b"LayoutReference" => {
                            item.layout_reference = get_attribute(&e, "name");
                        }
                        b"Label" => {
                            item.layout_reference =
                                Some(LayoutReferenceContainer::parse_label(reader, &e).unwrap());
                            depth -= 1;
                        }
                        b"Calculation" => {
                            item.layout_reference =
                                Calculation::from_xml(reader, &e).unwrap().display();
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
        let layout_reference = self
            .layout_reference
            .clone()
            .unwrap_or("🚨🚨🚨 <BROKEN REFERENCE> 🚨🚨🚨".to_string());

        if self.reference_type == "1" {
            Some(format!("Layout: <{layout_reference}>"))
        } else if self.reference_type == "3" {
            Some(format!("Layoutname: {layout_reference}"))
        } else if self.reference_type == "4" {
            Some(format!("Layoutnr.: {layout_reference}"))
        } else if self.reference_type == "5" {
            Some(format!("Layout: \"{layout_reference}\""))
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
        let xml = r#"
            <LayoutReferenceContainer value="1">
                <Label>Originallayout</Label>
            </LayoutReferenceContainer>
        "#;

        let mut reader = Reader::from_str(xml.trim());
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
        let xml = r#"
            <LayoutReferenceContainer value="5">
                <LayoutReference id="2" name="Aufgabenliste" UUID="EBFC887D-CF13-4ABE-BB71-78EDC2B30FEE"></LayoutReference>
            </LayoutReferenceContainer>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = Some(r#"Layout: "Aufgabenliste""#.to_string());
        assert_eq!(
            LayoutReferenceContainer::from_xml(&mut reader, &element)
                .unwrap()
                .display(),
            expected_output
        );
    }

    #[test]
    fn test_by_name() {
        let xml = r#"
            <LayoutReferenceContainer value="3">
                <Calculation datatype="1" position="5">
                    <Calculation>
                        <Text><![CDATA["LAYOUT_NAME_CALCULATION"]]></Text>
                        <ChunkList hash="56E1B7D90047D6C6C4421314C701E408">
                            <Chunk type="NoRef">&quot;LAYOUT_NAME_CALCULATION&quot;</Chunk>
                        </ChunkList>
                    </Calculation>
                </Calculation>
            </LayoutReferenceContainer>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = Some(r#"Layoutname: "LAYOUT_NAME_CALCULATION""#.to_string());
        assert_eq!(
            LayoutReferenceContainer::from_xml(&mut reader, &element)
                .unwrap()
                .display(),
            expected_output
        );
    }

    #[test]
    fn test_parameter_value_item() {
        let xml = r#"
            <Parameter type="LayoutReferenceContainer">
                <LayoutReferenceContainer value="5">
                    <LayoutReference id="8" name="Palettes"></LayoutReference>
                </LayoutReferenceContainer>
            </Parameter>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let expected_output = Some(r#"Layout: "Palettes""#.to_string());
        assert_eq!(
            LayoutReferenceContainer::from_xml(&mut reader, &element)
                .unwrap()
                .display(),
            expected_output
        );
    }
}
