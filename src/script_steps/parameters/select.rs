use crate::utils::xml_node::Node;

/// The window selection of a window reference, e.g. `Name: $window`
pub fn display(select: &Node) -> Option<String> {
    select.calculation().map(|name| format!("Name: {name}"))
}

#[cfg(test)]
mod tests {
    use crate::utils::xml_node::Node;

    use crate::script_steps::parameters::select;

    #[test]
    fn test() {
        let xml = r#"<Select kind="0" type="current"></Select>"#;

        let element = Node::parse(xml).unwrap();

        assert_eq!(select::display(&element), None);
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

        let element = Node::parse(xml).unwrap();

        assert_eq!(
            select::display(&element),
            Some("Name: $FensterName".to_string())
        );
    }
}
