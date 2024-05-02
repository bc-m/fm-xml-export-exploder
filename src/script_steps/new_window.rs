use quick_xml::events::Event;
use quick_xml::Reader;

use crate::script_steps::parameters::layout_reference::LayoutReferenceContainer;
use crate::utils::attributes::get_attribute;
use crate::utils::xml_utils;
use crate::utils::xml_utils::local_name_to_string;

#[derive(Debug, Default)]
struct WindowOptions {
    close: bool,
    minimize: bool,
    maximize: bool,
    resize: bool,
    menu_bar: bool,
    toolbar: bool,
    dim_parent_window: bool,
}

#[derive(Debug, Default)]
struct WindowReference {
    style: String,
    name: String,
    layout_reference: String,
    bounds: Bounds,
    options: WindowOptions,
}

#[derive(Debug, Default)]
struct Bounds {
    height: Option<String>,
    width: Option<String>,
    top: Option<String>,
    left: Option<String>,
}

#[derive(Debug, Default)]
struct XmlTrackingState {
    in_name_calc: bool,
    in_options: bool,
    in_close_option: bool,
    in_minimize_option: bool,
    in_maximize_option: bool,
    in_resize_option: bool,
    in_menu_bar_option: bool,
    in_toolbar_option: bool,
    in_dim_parent_window_option: bool,
    in_height_calculation: bool,
    in_width_calculation: bool,
    in_top_calculation: bool,
    in_left_calculation: bool,
}

const WINDOW_NAME_PATH: &str =
    "Step/ParameterValues/Parameter/WindowReference/Name/Calculation/Calculation/Text";

pub fn sanitize(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut window = WindowReference::default();
    let mut xml_pos = XmlTrackingState::default();
    let mut current_path = Vec::new();

    let mut reader = Reader::from_str(step);
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                current_path.push(local_name_to_string(e.name().as_ref()));
                match e.name().as_ref() {
                    b"Step" => {
                        name = get_attribute(&e, "name").unwrap().to_string();
                    }
                    b"Style" => {
                        window.style = get_attribute(&e, "name").unwrap().to_string();
                    }
                    b"LayoutReferenceContainer" => {
                        window.layout_reference =
                            LayoutReferenceContainer::from_xml(&mut reader, &e)
                                .unwrap()
                                .display()
                                .unwrap()
                    }
                    b"Options" => xml_pos.in_options = true,
                    b"Close" => xml_pos.in_close_option = true,
                    b"Minimize" => xml_pos.in_minimize_option = true,
                    b"Maximize" => xml_pos.in_maximize_option = true,
                    b"Resize" => xml_pos.in_resize_option = true,
                    b"MenuBar" => xml_pos.in_menu_bar_option = true,
                    b"Toolbar" => xml_pos.in_toolbar_option = true,
                    b"DimParentWindow" => xml_pos.in_dim_parent_window_option = true,
                    b"height" => xml_pos.in_height_calculation = true,
                    b"width" => xml_pos.in_width_calculation = true,
                    b"top" => xml_pos.in_top_calculation = true,
                    b"left" => xml_pos.in_left_calculation = true,
                    b"Text" => {
                        if current_path.join("/").as_str() == WINDOW_NAME_PATH {
                            xml_pos.in_name_calc = true;
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                if !xml_pos.in_options {
                    continue;
                }

                let bool = match e.unescape().unwrap().as_ref() {
                    "True" => true,
                    "False" => false,
                    _ => {
                        continue;
                    }
                };
                if xml_pos.in_close_option {
                    window.options.close = bool
                }
                if xml_pos.in_minimize_option {
                    window.options.minimize = bool
                }
                if xml_pos.in_maximize_option {
                    window.options.maximize = bool
                }
                if xml_pos.in_resize_option {
                    window.options.resize = bool
                }
                if xml_pos.in_menu_bar_option {
                    window.options.menu_bar = bool
                }
                if xml_pos.in_toolbar_option {
                    window.options.toolbar = bool
                }
                if xml_pos.in_dim_parent_window_option {
                    window.options.dim_parent_window = bool
                }
            }
            Ok(Event::CData(e)) => {
                if xml_pos.in_name_calc {
                    window.name = xml_utils::cdata_to_string(&e);
                }
                if xml_pos.in_height_calculation {
                    window.bounds.height = Some(xml_utils::cdata_to_string(&e));
                }
                if xml_pos.in_width_calculation {
                    window.bounds.width = Some(xml_utils::cdata_to_string(&e));
                }
                if xml_pos.in_top_calculation {
                    window.bounds.top = Some(xml_utils::cdata_to_string(&e));
                }
                if xml_pos.in_left_calculation {
                    window.bounds.left = Some(xml_utils::cdata_to_string(&e));
                }
            }
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"Options" => xml_pos.in_options = false,
                    b"Close" => xml_pos.in_close_option = false,
                    b"Minimize" => xml_pos.in_minimize_option = false,
                    b"Maximize" => xml_pos.in_maximize_option = false,
                    b"Resize" => xml_pos.in_resize_option = false,
                    b"MenuBar" => xml_pos.in_menu_bar_option = false,
                    b"Toolbar" => xml_pos.in_toolbar_option = false,
                    b"DimParentWindow" => xml_pos.in_dim_parent_window_option = false,
                    b"height" => xml_pos.in_height_calculation = false,
                    b"width" => xml_pos.in_width_calculation = false,
                    b"top" => xml_pos.in_top_calculation = false,
                    b"left" => xml_pos.in_left_calculation = false,
                    b"Calculation" => xml_pos.in_name_calc = false,
                    _ => {}
                }

                current_path.pop();
            }
            _ => {}
        }
        buf.clear()
    }

    let mut params: Vec<String> = Vec::new();

    if !window.style.is_empty() {
        let style_formatted = format!("Style: {}", window.style);
        params.push(style_formatted);
    }
    if !window.name.is_empty() {
        let calculation_formatted = format!("Name: {}", window.name);
        params.push(calculation_formatted);
    }
    if !window.layout_reference.is_empty() {
        params.push(window.layout_reference)
    }
    if window.bounds.height.is_some() {
        params.push(format!("Height: {}", window.bounds.height.unwrap()));
    }
    if window.bounds.width.is_some() {
        params.push(format!("Width: {}", window.bounds.width.unwrap()));
    }
    if window.bounds.top.is_some() {
        params.push(format!("Top: {}", window.bounds.top.unwrap()));
    }
    if window.bounds.left.is_some() {
        params.push(format!("Left: {}", window.bounds.left.unwrap()));
    }
    if !window.options.close {
        params.push("Close: OFF".to_string());
    }
    if !window.options.minimize {
        params.push("Minimize: OFF".to_string());
    }
    if !window.options.maximize {
        params.push("Maximize: OFF".to_string());
    }
    if !window.options.resize {
        params.push("Resize: OFF".to_string());
    }
    if !window.options.menu_bar {
        params.push("Menu: OFF".to_string());
    }
    if !window.options.toolbar {
        params.push("Toolbar: OFF".to_string());
    }

    if name.is_empty() {
        println!("empty primitive");
        None
    } else {
        Some(format!("{} [ {} ]", name, params.join(" ; ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let xml = r#"
            <Step index="1" id="122" name="Neues Fenster" enable="True">
                <UUID>0376F75D-7B1E-41B9-A6D5-C585D2862A6A</UUID>
                <OwnerID></OwnerID>
                <Options>0</Options>
                <ParameterValues membercount="1">
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
                </ParameterValues>
            </Step>
        "#;

        let expected_output =
            Some("Neues Fenster [ Style: Dokument ; Layout: <Originallayout> ]".to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }

    #[test]
    fn test_window_size() {
        let xml = r#"
            <Step index="13" id="122" name="Neues Fenster" enable="True">
                <UUID>0612C406-BCCF-4636-946B-2EEB14EE69CF</UUID>
                <OwnerID></OwnerID>
                <Options>16392</Options>
                <ParameterValues membercount="1">
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
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(r#"Neues Fenster [ Style: Dokument ; Name: "Foo Bar" ; Layout: <Originallayout> ; Height: 100 ; Width: 200 ; Top: 300 ; Left: 400 ; Minimize: OFF ; Maximize: OFF ; Resize: OFF ; Menu: OFF ; Toolbar: OFF ]"#.to_string());
        assert_eq!(sanitize(xml.trim()), expected_output);
    }
}
