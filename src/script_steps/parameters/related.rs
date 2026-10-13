use crate::script_steps::parameters::format::BROKEN_REFERENCE_NAME;
use crate::script_steps::parameters::layout_reference;
use crate::script_steps::parameters::window_reference;
use crate::utils::xml_node::Node;

/// Go to Related Record, e.g.
/// `Table: Contacts ; Layout: "Contact" ; New window ; Style: Card ; Show related`
pub fn display(parameter: &Node) -> String {
    let mut parts = Vec::new();
    visit(parameter, &mut parts);
    parts.join(" ; ")
}

fn visit(node: &Node, parts: &mut Vec<String>) {
    for child in &node.children {
        match child.name.as_str() {
            "TableOccurrenceReference" => {
                let table_occurrence = child.attr("name").unwrap_or(BROKEN_REFERENCE_NAME);
                parts.push(format!("Table: {table_occurrence}"));
            }
            "LayoutReferenceContainer" => {
                parts.push(layout_reference::display(child).unwrap_or_default());
            }
            "WindowReference" => {
                parts.push("New window".to_string());
                // The new window's layout only repeats the target layout
                let window = window_reference::display_without_layout(child);
                if !window.is_empty() {
                    parts.push(window);
                }
            }
            _ => {
                if child.name == "Options" {
                    if child.attr_is_true("ShowRelated") {
                        parts.push("Show related".to_string());
                    }
                    if child.attr_is_true("matchFoundSet") {
                        parts.push("Match found set".to_string());
                    }
                }
                visit(child, parts);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::xml_node::Node;

    use crate::script_steps::parameters::related;

    #[test]
    fn new_window_does_not_repeat_layout() {
        let xml = r#"<Parameter type="Related"><TableOccurrenceReference id="1" name="Contacts"></TableOccurrenceReference><LayoutReferenceContainer value="1"><Label>original layout</Label></LayoutReferenceContainer><WindowReference><Style name="Card" value="1"></Style><Name><Calculation datatype="1" position="0"><Calculation><Text><![CDATA["Detail"]]></Text></Calculation></Calculation></Name><LayoutReferenceContainer value="1"><Label>original layout</Label></LayoutReferenceContainer></WindowReference><Options matchFoundSet="True" ShowRelated="True"></Options></Parameter>"#;
        let e = Node::parse(xml).unwrap();
        assert_eq!(
            related::display(&e),
            r#"Table: Contacts ; Layout: <original layout> ; New window ; Style: Card ; Name: "Detail" ; Show related ; Match found set"#
        );
    }
}
