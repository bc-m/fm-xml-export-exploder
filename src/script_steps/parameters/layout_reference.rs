use crate::utils::xml_node::Node;

/// e.g. `Layout: "Invoices"`, `Layout: <original layout>` or `Layoutname: $name`
pub fn display(container: &Node) -> Option<String> {
    let mut reference_type = container.attr("value").unwrap_or_default();
    let mut layout_reference = None;
    for child in container.descendants().into_iter().skip(1) {
        match child.name.as_str() {
            "LayoutReferenceContainer" => reference_type = child.attr("value").unwrap_or_default(),
            "LayoutReference" => layout_reference = child.attr("name").map(str::to_string),
            "Label" => layout_reference = Some(child.raw_text.clone()),
            "Calculation" => layout_reference = child.calculation(),
            _ => {}
        }
    }

    let layout_reference = layout_reference
        .as_deref()
        .unwrap_or("🚨🚨🚨 <BROKEN REFERENCE> 🚨🚨🚨");

    match reference_type {
        "1" => Some(format!("Layout: <{layout_reference}>")),
        "3" => Some(format!("Layoutname: {layout_reference}")),
        "4" => Some(format!("Layoutnr.: {layout_reference}")),
        "5" => Some(format!("Layout: \"{layout_reference}\"")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::xml_node::Node;

    #[test]
    fn test() {
        let xml = r#"
            <LayoutReferenceContainer value="1">
                <Label>Originallayout</Label>
            </LayoutReferenceContainer>
        "#;

        let element = Node::parse(xml).unwrap();

        let expected_output = Some("Layout: <Originallayout>".to_string());
        assert_eq!(display(&element), expected_output);
    }

    #[test]
    fn test_layout() {
        let xml = r#"
            <LayoutReferenceContainer value="5">
                <LayoutReference id="2" name="Aufgabenliste" UUID="EBFC887D-CF13-4ABE-BB71-78EDC2B30FEE"></LayoutReference>
            </LayoutReferenceContainer>
        "#;

        let element = Node::parse(xml).unwrap();

        let expected_output = Some(r#"Layout: "Aufgabenliste""#.to_string());
        assert_eq!(display(&element), expected_output);
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

        let element = Node::parse(xml).unwrap();

        let expected_output = Some(r#"Layoutname: "LAYOUT_NAME_CALCULATION""#.to_string());
        assert_eq!(display(&element), expected_output);
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

        let element = Node::parse(xml).unwrap();

        let expected_output = Some(r#"Layout: "Palettes""#.to_string());
        assert_eq!(display(&element), expected_output);
    }
}
