use crate::script_steps::parameters::{boolean_container, layout_reference};
use crate::utils::xml_node::Node;

/// e.g. `Style: Document ; Name: $name ; Height: 400 ; Close: OFF`
pub fn display(parameter: &Node) -> String {
    let mut parts = Vec::new();
    visit(parameter, true, &mut parts);
    parts.join(" ; ")
}

/// For the new window of Go to Related Record, whose nested layout only repeats
/// the step's target layout.
pub fn display_without_layout(parameter: &Node) -> String {
    let mut parts = Vec::new();
    visit(parameter, false, &mut parts);
    parts.join(" ; ")
}

fn visit(node: &Node, include_layout: bool, parts: &mut Vec<String>) {
    for child in &node.children {
        let parsed = match child.name.as_str() {
            "Style" => child.attr("name").map(|style| format!("Style: {style}")),
            "LayoutReferenceContainer" => {
                let layout = layout_reference::display(child);
                include_layout.then(|| layout.unwrap_or_default())
            }
            // The window to act on, e.g. `Name: $window`
            "Select" => child.calculation().map(|name| format!("Name: {name}")),
            // Set Window Title
            "Rename" => child
                .calculation()
                .map(|title| format!("New Title: {title}")),
            "Name" | "height" | "width" | "top" | "left" | "Text" => child
                .calculation()
                .map(|calc| format!("{}: {calc}", capitalize(&child.name))),
            "Close" | "Minimize" | "Maximize" | "Resize" | "MenuBar" | "Toolbar"
            | "DimParentWindow" => boolean_container::display(child),
            _ => {
                visit(child, include_layout, parts);
                continue;
            }
        };
        parts.extend(parsed);
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::xml_node::Node;

    use crate::script_steps::parameters::window_reference;

    #[test]
    fn style() {
        let element = Node::parse(
            r#"<WindowReference><Style name="Dokument" value="3221291010"></Style></WindowReference>"#,
        )
        .unwrap();
        assert_eq!(window_reference::display(&element), "Style: Dokument");
    }

    #[test]
    fn current_window_is_not_shown() {
        let element = Node::parse(
            r#"<WindowReference><Select kind="0" type="current"></Select></WindowReference>"#,
        )
        .unwrap();
        assert_eq!(window_reference::display(&element), "");
    }

    #[test]
    fn window_by_name() {
        let element = Node::parse(
            r#"<WindowReference><Select kind="1" type="Calculated"><Name current="True"><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[$FensterName]]></Text></Calculation></Calculation></Name></Select></WindowReference>"#,
        )
        .unwrap();
        assert_eq!(window_reference::display(&element), "Name: $FensterName");
    }

    #[test]
    fn test() {
        let xml = r#"
            <Parameter type="WindowReference">
                <WindowReference>
                    <Style name="Dokument" value="3606018"></Style>
                    <Name></Name>
                    <LayoutReferenceContainer value="1">
                        <Label>Originallayout</Label>
                    </LayoutReferenceContainer>
                    <Bounds>
                        <height></height>
                        <width></width>
                        <top></top>
                        <left></left>
                    </Bounds>
                    <Options value="3606018">
                        <Close>True</Close>
                        <Minimize>True</Minimize>
                        <Maximize>True</Maximize>
                        <Resize>True</Resize>
                        <MenuBar>True</MenuBar>
                        <Toolbar>True</Toolbar>
                        <DimParentWindow>True</DimParentWindow>
                    </Options>
                </WindowReference>
            </Parameter>
        "#;

        let element = Node::parse(xml).unwrap();
        let expected_output = "Style: Dokument ; Layout: <Originallayout>";
        assert_eq!(window_reference::display(&element), expected_output);
    }

    #[test]
    fn test_window_size() {
        let xml = r#"
            <Parameter type="WindowReference">
                <WindowReference>
                    <Style name="Dokument" value="3221291010"></Style>
                    <Name>
                        <Calculation datatype="1" position="0">
                            <Calculation>
                                <Text><![CDATA["Foo Bar"]]></Text>
                                <ChunkList hash="721B1A57045045B02A87AE94CE6EBD08">
                                    <Chunk type="NoRef">&quot;Foo Bar&quot;</Chunk>
                                </ChunkList>
                            </Calculation>
                        </Calculation>
                    </Name>
                    <LayoutReferenceContainer value="1">
                        <Label>Originallayout</Label>
                    </LayoutReferenceContainer>
                    <Bounds>
                        <height>
                            <Calculation datatype="1" position="1">
                                <Calculation>
                                    <Text><![CDATA[100]]></Text>
                                    <ChunkList hash="45A530D85F1E3C1F21F81650ACF37719">
                                        <Chunk type="NoRef">100</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </height>
                        <width>
                            <Calculation datatype="1" position="2">
                                <Calculation>
                                    <Text><![CDATA[200]]></Text>
                                    <ChunkList hash="8BE34A566DA016E82241E5F1CBBB81F3">
                                        <Chunk type="NoRef">200</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </width>
                        <top>
                            <Calculation datatype="1" position="3">
                                <Calculation>
                                    <Text><![CDATA[300]]></Text>
                                    <ChunkList hash="699252F1918300A60EF02F519F7447C7">
                                        <Chunk type="NoRef">300</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </top>
                        <left>
                            <Calculation datatype="1" position="4">
                                <Calculation>
                                    <Text><![CDATA[400]]></Text>
                                    <ChunkList hash="CAE3589A1895AE0560BE789B6E9902C1">
                                        <Chunk type="NoRef">400</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </left>
                    </Bounds>
                    <Options value="3221291010">
                        <Close>True</Close>
                        <Minimize>False</Minimize>
                        <Maximize>False</Maximize>
                        <Resize>False</Resize>
                        <MenuBar>False</MenuBar>
                        <Toolbar>False</Toolbar>
                        <DimParentWindow>False</DimParentWindow>
                    </Options>
                </WindowReference>
            </Parameter>
        "#;

        let element = Node::parse(xml).unwrap();
        let expected_output = r#"Style: Dokument ; Name: "Foo Bar" ; Layout: <Originallayout> ; Height: 100 ; Width: 200 ; Top: 300 ; Left: 400 ; Minimize: OFF ; Maximize: OFF ; Resize: OFF ; Menu: OFF ; Toolbar: OFF"#;
        assert_eq!(window_reference::display(&element), expected_output);
    }

    #[test]
    fn test_adjust_window() {
        let xml = r#"
           <Parameter type="WindowReference">
                 <WindowReference>
                    <Select kind="0" type="current"></Select>
                    <Bounds>
                        <height>
                            <Calculation datatype="1" position="1">
                                <Calculation>
                                    <Text><![CDATA[$$AppWindowHeight]]></Text>
                                    <ChunkList hash="0962B8365B23C47B6916CAF58F345C32">
                                        <Chunk type="VariableReference">$$AppWindowHeight</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </height>
                        <width>
                            <Calculation datatype="1" position="2">
                                <Calculation>
                                    <Text><![CDATA[$$AppWindowWidth]]></Text>
                                    <ChunkList hash="B951C770B65DCD808B8D64E1873DF204">
                                        <Chunk type="VariableReference">$$AppWindowWidth</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </width>
                        <top>
                            <Calculation datatype="1" position="3">
                                <Calculation>
                                    <Text><![CDATA[0]]></Text>
                                    <ChunkList hash="9BF55A5D51230A8EA20D4BAA0F468EEC">
                                        <Chunk type="NoRef">0</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </top>
                        <left>
                            <Calculation datatype="1" position="4">
                                <Calculation>
                                    <Text><![CDATA[0]]></Text>
                                    <ChunkList hash="9BF55A5D51230A8EA20D4BAA0F468EEC">
                                        <Chunk type="NoRef">0</Chunk>
                                    </ChunkList>
                                </Calculation>
                            </Calculation>
                        </left>
                    </Bounds>
                </WindowReference>
           </Parameter>
        "#;

        let element = Node::parse(xml).unwrap();
        let expected_output =
            r#"Height: $$AppWindowHeight ; Width: $$AppWindowWidth ; Top: 0 ; Left: 0"#;
        assert_eq!(window_reference::display(&element), expected_output);
    }

    #[test]
    fn test_set_window_title() {
        let xml = r#"<Parameter type="WindowReference"><WindowReference><Select kind="1" type="Calculated"><Name current="True"><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[$old]]></Text></Calculation></Calculation></Name></Select><Rename><Calculation datatype="1" position="1"><Calculation><Text><![CDATA[$new]]></Text></Calculation></Calculation></Rename></WindowReference></Parameter>"#;
        let e = Node::parse(xml).unwrap();
        assert_eq!(
            window_reference::display(&e),
            "Name: $old ; New Title: $new"
        );
    }
}
