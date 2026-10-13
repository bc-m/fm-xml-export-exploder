use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::utils::attributes::{get_attributes, unescape_value};
use crate::utils::xml_utils::{cdata_to_string, general_ref_to_string, text_to_string};

/// Whether [`Node::walk`] continues into a node's children.
pub enum Visit {
    Descend,
    Skip,
}

/// A lightweight in-memory element tree.
///
/// Script steps are small, so each one is read into a tree once and all formatting
/// works on the tree instead of on a streaming reader.
#[derive(Debug, Default)]
pub struct Node {
    pub name: String,
    /// Attribute values with XML entities resolved
    pub attributes: Vec<(String, String)>,
    /// Text, entity references and CDATA of this element (not of its children), untrimmed
    pub raw_text: String,
    /// The CDATA sections of this element, in document order
    pub cdata: Vec<String>,
    pub children: Vec<Node>,
}

impl Node {
    /// Parses the first element of `xml`, e.g. a single `<Step>`.
    pub fn parse(xml: &str) -> Option<Node> {
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => return Some(Node::from_xml(&mut reader, &e)),
                Ok(Event::Empty(e)) => return Some(Node::new(&e)),
                Ok(Event::Eof) => return None,
                _ => {}
            }
            buf.clear();
        }
    }

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
                Ok(Event::Text(e)) => node.raw_text.push_str(&text_to_string(&e)),
                Ok(Event::CData(e)) => {
                    let cdata = cdata_to_string(&e);
                    node.raw_text.push_str(&cdata);
                    node.cdata.push(cdata);
                }
                Ok(Event::GeneralRef(e)) => {
                    node.raw_text.push_str(&general_ref_to_string(&e, false));
                }
                _ => {}
            }
            buf.clear();
        }
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

    /// The trimmed text content of this element (without its children).
    pub fn text(&self) -> &str {
        self.raw_text.trim()
    }

    pub fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|child| child.name == name)
    }

    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Node> {
        self.children.iter().filter(move |child| child.name == name)
    }

    /// This node and all nodes below it, in document order.
    pub fn descendants(&self) -> Vec<&Node> {
        let mut nodes = vec![self];
        for child in &self.children {
            nodes.extend(child.descendants());
        }
        nodes
    }

    /// Visits all nodes below this one in document order. Returning [`Visit::Skip`]
    /// leaves out that node's children, e.g. once a calculation has been read.
    pub fn walk<'a>(&'a self, visit: &mut impl FnMut(&'a Node) -> Visit) {
        for child in &self.children {
            if let Visit::Descend = visit(child) {
                child.walk(visit);
            }
        }
    }

    /// The first node named `name` at or below this node.
    pub fn descendant(&self, name: &str) -> Option<&Node> {
        if self.name == name {
            return Some(self);
        }
        self.children
            .iter()
            .find_map(|child| child.descendant(name))
    }

    /// The last node named `name` at or below this node.
    pub fn last_descendant(&self, name: &str) -> Option<&Node> {
        self.descendants()
            .into_iter()
            .rev()
            .find(|n| n.name == name)
    }

    /// The CDATA sections of all `<Text>` elements at or below this node, in document order.
    pub fn text_cdata(&self) -> Vec<&str> {
        self.descendants()
            .into_iter()
            .filter(|n| n.name == "Text")
            .flat_map(|n| n.cdata.iter().map(String::as_str))
            .collect()
    }

    /// The calculation text at or below this node, if not empty.
    ///
    /// FileMaker wraps calculations as `<Calculation><Calculation><Text><![CDATA[...]]>`;
    /// when there are several, the last one wins.
    pub fn calculation(&self) -> Option<String> {
        self.text_cdata()
            .last()
            .filter(|text| !text.is_empty())
            .map(|text| text.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tree() {
        let node = Node::parse(
            r#"<Options type="Title" name="&lt;Missing&gt;"><Title><Calculation datatype="1"><Calculation><Text><![CDATA[$Title & "&"]]></Text></Calculation></Calculation></Title></Options>"#,
        )
        .unwrap();
        assert_eq!(node.name, "Options");
        assert_eq!(node.attr("type"), Some("Title"));
        assert_eq!(node.attr("name"), Some("<Missing>"));
        assert_eq!(node.calculation().as_deref(), Some(r#"$Title & "&""#));
    }

    #[test]
    fn calculation_keeps_whitespace_and_takes_last() {
        let node = Node::parse(
            "<P><Calculation><Text><![CDATA[ first ]]></Text></Calculation><Calculation><Text><![CDATA[ last\n]]></Text></Calculation></P>",
        )
        .unwrap();
        assert_eq!(node.calculation().as_deref(), Some(" last\n"));
    }

    #[test]
    fn empty_calculation() {
        let node =
            Node::parse("<P><Calculation><Text><![CDATA[]]></Text></Calculation></P>").unwrap();
        assert_eq!(node.calculation(), None);
    }

    #[test]
    fn text_with_entities() {
        let node = Node::parse("<Label> original &amp; layout </Label>").unwrap();
        assert_eq!(node.raw_text, " original & layout ");
        assert_eq!(node.text(), "original & layout");
    }

    #[test]
    fn walk_can_skip_children() {
        let node =
            Node::parse("<Step><Calculation><Calculation/></Calculation><Name/></Step>").unwrap();
        let mut visited = Vec::new();
        node.walk(&mut |n| {
            visited.push(n.name.clone());
            if n.name == "Calculation" {
                Visit::Skip
            } else {
                Visit::Descend
            }
        });
        assert_eq!(visited, ["Calculation", "Name"]);
    }

    #[test]
    fn last_descendant() {
        let node = Node::parse(r#"<P><List name="a"><List name="b"/></List><List name="c"/></P>"#)
            .unwrap();
        assert_eq!(
            node.last_descendant("List").and_then(|n| n.attr("name")),
            Some("c")
        );
        assert_eq!(
            node.descendant("List").and_then(|n| n.attr("name")),
            Some("a")
        );
    }
}
