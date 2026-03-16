use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::script_steps::parameters::calculation::Calculation;
use crate::script_steps::parameters::target::Target;
use crate::utils::attributes::get_attribute;

#[derive(Debug, Default)]
pub struct DialogField {
    pub target: Option<String>,
    pub label: Option<String>,
    pub password: bool,
}

impl DialogField {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _: &BytesStart) -> DialogField {
        let mut item = DialogField::default();
        let mut depth = 1;

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(inner)) => {
                    depth += 1;
                    match inner.name().as_ref() {
                        b"Parameter" => match get_attribute(&inner, "type").as_deref() {
                            Some("Target") => {
                                item.target = Target::from_xml(reader, &inner).display();
                                depth -= 1;
                            }
                            Some("Label") => {
                                item.label = Calculation::from_xml(reader, &inner).display();
                                depth -= 1;
                            }
                            _ => {}
                        },
                        b"Boolean" => {
                            item.password = get_attribute(&inner, "type").as_deref()
                                == Some("Password")
                                && get_attribute(&inner, "value").as_deref() == Some("True");
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

        item
    }

    pub fn display(self, field_type: &str) -> Option<String> {
        let target = self.target?;

        let number = match field_type {
            "Field1" => "1",
            "Field2" => "2",
            "Field3" => "3",
            _ => "?",
        };

        let mut parts = vec![format!("Input {number}: {target}")];

        if let Some(label) = self.label
            && !label.is_empty()
        {
            parts.push(format!("Label {number}: {label}"));
        }

        if self.password {
            parts.push("Password".to_string());
        }

        Some(parts.join(" ; "))
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    use super::DialogField;

    #[test]
    fn test_field_with_variable_and_label() {
        let xml = r#"
            <Parameter type="Field1">
                <Parameter type="Target">
                    <Variable value="$input1">
                        <repetition>
                            <Calculation datatype="1" position="32">
                                <Calculation>
                                    <Text><![CDATA[1]]></Text>
                                </Calculation>
                            </Calculation>
                        </repetition>
                    </Variable>
                </Parameter>
                <Boolean type="Password" value="False"></Boolean>
                <Parameter type="Label">
                    <Calculation datatype="1" position="2">
                        <Calculation>
                            <Text><![CDATA["label1"]]></Text>
                        </Calculation>
                    </Calculation>
                </Parameter>
            </Parameter>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let field = DialogField::from_xml(&mut reader, &element);
        assert_eq!(field.target, Some("$input1".to_string()));
        assert_eq!(field.label, Some(r#""label1""#.to_string()));
        assert!(!field.password);
        assert_eq!(
            field.display("Field1"),
            Some(r#"Input 1: $input1 ; Label 1: "label1""#.to_string())
        );
    }

    #[test]
    fn test_field_with_password() {
        let xml = r#"
            <Parameter type="Field1">
                <Parameter type="Target">
                    <Variable value="$input1">
                        <repetition>
                            <Calculation datatype="1" position="32">
                                <Calculation>
                                    <Text><![CDATA[1]]></Text>
                                </Calculation>
                            </Calculation>
                        </repetition>
                    </Variable>
                </Parameter>
                <Boolean type="Password" value="True"></Boolean>
                <Parameter type="Label">
                    <Calculation datatype="1" position="2">
                        <Calculation>
                            <Text><![CDATA["label1"]]></Text>
                        </Calculation>
                    </Calculation>
                </Parameter>
            </Parameter>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let field = DialogField::from_xml(&mut reader, &element);
        assert!(field.password);
        assert_eq!(
            field.display("Field1"),
            Some(r#"Input 1: $input1 ; Label 1: "label1" ; Password"#.to_string())
        );
    }

    #[test]
    fn test_field2_display() {
        let xml = r#"
            <Parameter type="Field2">
                <Parameter type="Target">
                    <Variable value="$input2">
                        <repetition>
                            <Calculation datatype="1" position="33">
                                <Calculation>
                                    <Text><![CDATA[1]]></Text>
                                </Calculation>
                            </Calculation>
                        </repetition>
                    </Variable>
                </Parameter>
                <Boolean type="Password" value="False"></Boolean>
                <Parameter type="Label">
                    <Calculation datatype="1" position="3">
                        <Calculation>
                            <Text><![CDATA["second"]]></Text>
                        </Calculation>
                    </Calculation>
                </Parameter>
            </Parameter>
        "#;

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };

        let field = DialogField::from_xml(&mut reader, &element);
        assert_eq!(
            field.display("Field2"),
            Some(r#"Input 2: $input2 ; Label 2: "second""#.to_string())
        );
    }
}
