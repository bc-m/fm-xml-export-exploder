use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::script_steps::parameters::boolean_container::BooleanContainer;
use crate::script_steps::parameters::calculation::Calculation;
use crate::script_steps::parameters::layout_reference::LayoutReferenceContainer;
use crate::script_steps::parameters::select::Select;
use crate::script_steps::parameters::style::Style;
use crate::utils::xml_utils::local_name_to_string;

#[derive(Debug, Default)]
pub struct WindowReference {
    pub parameters: Vec<String>,
}

pub fn first_char_uppercase(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

impl WindowReference {
    pub fn from_xml(reader: &mut Reader<&[u8]>, _: &BytesStart) -> Option<WindowReference> {
        let mut depth = 1;
        let mut window_reference = WindowReference {
            parameters: Vec::new(),
        };

        let mut buf: Vec<u8> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Err(_) => continue,
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    depth += 1;

                    let element_name = e.name();
                    match element_name.as_ref() {
                        b"Style" => {
                            if let Ok(param_value) = Style::from_xml(reader, &e) {
                                if let Some(display) = param_value.display() {
                                    window_reference.parameters.push(display);
                                }
                            }
                            depth -= 1;
                        }
                        b"LayoutReferenceContainer" => {
                            window_reference.parameters.push(
                                LayoutReferenceContainer::from_xml(reader, &e)
                                    .unwrap()
                                    .display()
                                    .unwrap_or("".to_string()),
                            );
                            depth -= 1;
                        }
                        b"Select" => {
                            if let Ok(param_value) = Select::from_xml(reader, &e) {
                                if let Some(calc) = param_value.display() {
                                    window_reference.parameters.push(calc);
                                }
                            }
                            depth -= 1;
                        }
                        b"Name" | b"height" | b"width" | b"top" | b"left" | b"Text" => {
                            if let Ok(param_value) = Calculation::from_xml(reader, &e) {
                                if let Some(calc) = param_value.display() {
                                    let name = local_name_to_string(element_name.as_ref());
                                    let name = first_char_uppercase(name.as_str());
                                    window_reference
                                        .parameters
                                        .push(format!("{}: {}", name, calc));
                                }
                            }
                            depth -= 1;
                        }
                        b"Close" | b"Minimize" | b"Maximize" | b"Resize" | b"MenuBar"
                        | b"Toolbar" | b"DimParentWindow" => {
                            if let Ok(param_value) = BooleanContainer::from_xml(reader, &e) {
                                if let Some(calc) = param_value.display() {
                                    window_reference.parameters.push(calc);
                                }
                            }
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
            buf.clear()
        }

        Some(window_reference)
    }

    pub fn display(&self) -> Option<String> {
        Some(self.parameters.join(" ; "))
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    use crate::script_steps::parameters::window_reference::WindowReference;

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

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };
        let expected_output = Some("Style: Dokument ; Layout: <Originallayout>".to_string());
        assert_eq!(
            WindowReference::from_xml(&mut reader, &element)
                .unwrap()
                .display(),
            expected_output
        );
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

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };
        let expected_output = Some(r#"Style: Dokument ; Name: "Foo Bar" ; Layout: <Originallayout> ; Height: 100 ; Width: 200 ; Top: 300 ; Left: 400 ; Minimize: OFF ; Maximize: OFF ; Resize: OFF ; Menu: OFF ; Toolbar: OFF"#.to_string());
        assert_eq!(
            WindowReference::from_xml(&mut reader, &element)
                .unwrap()
                .display(),
            expected_output
        );
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

        let mut reader = Reader::from_str(xml.trim());
        let element = match reader.read_event() {
            Ok(Event::Start(e)) => e,
            _ => panic!("Wrong read event"),
        };
        let expected_output = Some(
            r#"Height: $$AppWindowHeight ; Width: $$AppWindowWidth ; Top: 0 ; Left: 0"#.to_string(),
        );
        assert_eq!(
            WindowReference::from_xml(&mut reader, &element)
                .unwrap()
                .display(),
            expected_output
        );
    }
}
