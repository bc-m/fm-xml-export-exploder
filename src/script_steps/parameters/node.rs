use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::utils::attributes::{get_attributes, unescape_value};
use crate::utils::xml_utils::{cdata_to_string, general_ref_to_string, text_to_string};

/// A lightweight in-memory element tree for parameters with deeply nested
/// structures, where a streaming parser would get unwieldy.
#[derive(Debug, Default)]
pub struct Node {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub text: String,
    pub children: Vec<Node>,
}

impl Node {
    /// Reads the element started by `start` up to and including its end tag.
    pub fn from_xml(reader: &mut Reader<&[u8]>, start: &BytesStart) -> Node {
        let mut node = Node::new(start);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) | Ok(Event::End(_)) => break,
                Ok(Event::Start(e)) => {
                    let child = Node::from_xml(reader, &e);
                    node.children.push(child);
                }
                Ok(Event::Empty(e)) => node.children.push(Node::new(&e)),
                Ok(Event::Text(e)) => node.text.push_str(&text_to_string(&e)),
                Ok(Event::CData(e)) => node.text.push_str(&cdata_to_string(&e)),
                Ok(Event::GeneralRef(e)) => node.text.push_str(&general_ref_to_string(&e, false)),
                _ => {}
            }
            buf.clear();
        }
        node.text = node.text.trim().to_string();
        node
    }

    fn new(start: &BytesStart) -> Node {
        Node {
            name: start.name().as_ref().to_owned(),
            attributes: get_attributes(start)
                .into_iter()
                .map(|(key, value)| (key, unescape_value(&value)))
                .collect(),
            ..Default::default()
        }
    }

    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    pub fn attr_is_true(&self, name: &str) -> bool {
        self.attr(name) == Some("True")
    }

    pub fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|child| child.name == name)
    }

    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Node> {
        self.children.iter().filter(move |child| child.name == name)
    }

    /// The first node named `name` at or below this node (depth-first).
    pub fn descendant(&self, name: &str) -> Option<&Node> {
        if self.name == name {
            return Some(self);
        }
        self.children
            .iter()
            .find_map(|child| child.descendant(name))
    }

    /// The text of the first `<Calculation>` at or below this node, if not empty.
    pub fn calculation(&self) -> Option<String> {
        if self.name == "Calculation" {
            return match self.child("Calculation") {
                Some(inner) => inner.calculation(),
                None => self
                    .child("Text")
                    .map(|text| text.text.clone())
                    .filter(|text| !text.is_empty()),
            };
        }
        self.children.iter().find_map(Node::calculation)
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    use super::*;

    #[test]
    fn test() {
        let xml = r#"<Options type="Title" name="&lt;Missing&gt;"><Title><Calculation datatype="1"><Calculation><Text><![CDATA[$Title & "&"]]></Text></Calculation></Calculation></Title></Options>"#;
        let mut reader = Reader::from_str(xml);
        let Ok(Event::Start(e)) = reader.read_event() else {
            panic!("Wrong read event");
        };
        let node = Node::from_xml(&mut reader, &e);
        assert_eq!(node.name, "Options");
        assert_eq!(node.attr("type"), Some("Title"));
        assert_eq!(node.attr("name"), Some("<Missing>"));
        assert_eq!(node.calculation().as_deref(), Some(r#"$Title & "&""#));
        assert!(matches!(reader.read_event(), Ok(Event::Eof)));
    }
}
