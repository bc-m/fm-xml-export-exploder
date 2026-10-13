use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::script_steps::parameters::calculation::Calculation;
use crate::utils::attributes::parse_unescaped_attribute;

#[derive(Debug, Default)]
pub struct ScriptReference {
    pub data_source_name: Option<String>,
    pub script_name: Option<String>,
    /// Install OnTimer Script stores its script parameter inside the reference
    pub parameter: Option<String>,
}

impl ScriptReference {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _: &BytesStart) -> ScriptReference {
        let mut depth = 1;
        let mut item = ScriptReference::default();

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    match e.name().as_ref() {
                        "DataSourceReference" => {
                            item.data_source_name = parse_unescaped_attribute(&e, "name")
                        }
                        "ScriptReference" => {
                            item.script_name = parse_unescaped_attribute(&e, "name")
                        }
                        "repetition" => {
                            item.parameter = Calculation::from_xml(reader, &e).display();
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

        item
    }

    pub fn display(self) -> Option<String> {
        let mut parts = Vec::new();
        if let Some(name) = self.script_name {
            parts.push(format!("\"{name}\""));
        }
        if let Some(name) = self.data_source_name {
            parts.push(format!("from file \"{name}\""));
        }
        let mut display = parts.join(" ");
        if let Some(parameter) = self.parameter {
            if !display.is_empty() {
                display.push_str(" ; ");
            }
            display.push_str(&format!("Parameter: {parameter}"));
        }
        (!display.is_empty()).then_some(display)
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    use super::ScriptReference;

    #[test]
    fn script_with_parameter() {
        let xml = r#"<Parameter type="ScriptReference"><ScriptReference id="9" name="All script steps"><repetition><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[$Param]]></Text></Calculation></Calculation></repetition></ScriptReference></Parameter>"#;
        let mut reader = Reader::from_str(xml);
        let Ok(Event::Start(e)) = reader.read_event() else {
            panic!("Wrong read event");
        };
        assert_eq!(
            ScriptReference::from_xml(&mut reader, &e)
                .display()
                .as_deref(),
            Some(r#""All script steps" ; Parameter: $Param"#)
        );
    }
}
