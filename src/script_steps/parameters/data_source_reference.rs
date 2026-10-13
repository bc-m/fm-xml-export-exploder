use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::script_steps::parameters::node::Node;
use crate::utils::attributes::parse_unescaped_attribute;

#[derive(Debug, Default)]
pub struct DataSourceReference {
    pub id: Option<String>,
    pub name: Option<String>,
    /// XML data source of Convert File / Import Records, e.g. `XML: "http:xml" ; XSL: "$path"`
    pub xml_source: Vec<String>,
}

impl DataSourceReference {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _: &BytesStart) -> DataSourceReference {
        let mut depth = 1;
        let mut item = DataSourceReference::default();

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;
                    match e.name().as_ref() {
                        "DataSourceReference" => {
                            item.id = parse_unescaped_attribute(&e, "id");
                            item.name = parse_unescaped_attribute(&e, "name");
                        }
                        label @ ("XML" | "XSL") => {
                            let label = label.to_string();
                            let source = Node::from_xml(reader, &e);
                            depth -= 1;
                            let value = source.calculation().or_else(|| {
                                let paths: Vec<String> = source
                                    .descendant("ObjectList")?
                                    .children_named("Location")
                                    .map(|location| location.text.clone())
                                    .collect();
                                (!paths.is_empty()).then(|| format!("\"{}\"", paths.join("\n")))
                            });
                            if let Some(value) = value {
                                item.xml_source.push(format!("{label}: {value}"));
                            }
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
        let mut parts: Vec<String> = self
            .name
            .map(|name| {
                if self.id.as_deref() == Some("0") {
                    name
                } else {
                    format!("\"{name}\"")
                }
            })
            .into_iter()
            .collect();
        parts.extend(self.xml_source);
        (!parts.is_empty()).then(|| parts.join(" ; "))
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    use super::DataSourceReference;

    #[test]
    fn xml_source() {
        let xml = r#"<Parameter type="DataSourceReference"><DataSourceReference fileType="XML " isPredefined="False" delimiter="4" value="4"><XML><Calculation><Text><![CDATA["http:xml"]]></Text></Calculation></XML><XSL><UniversalPathList fileType="XML " membercount="1"><ObjectList><Location><![CDATA[$PathXSLT]]></Location></ObjectList></UniversalPathList></XSL></DataSourceReference></Parameter>"#;
        let mut reader = Reader::from_str(xml);
        let Ok(Event::Start(e)) = reader.read_event() else {
            panic!("Wrong read event");
        };
        assert_eq!(
            DataSourceReference::from_xml(&mut reader, &e)
                .display()
                .as_deref(),
            Some(r#"XML: "http:xml" ; XSL: "$PathXSLT""#)
        );
    }
}
