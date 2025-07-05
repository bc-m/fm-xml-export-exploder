use quick_xml::events::Event;
use quick_xml::Reader;

use crate::script_steps::constants::{id_to_script_step, ScriptStep};
use crate::script_steps::parameters::parameter_values::ParameterValues;
use crate::utils::attributes::get_attribute;

pub fn from_xml(step_id: &u32, step: &str) -> Option<String> {
    let mut name = String::new();
    let mut parameters: Vec<String> = Vec::new();

    let mut reader = Reader::from_str(step);
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => {
                    if let Some(value) = get_attribute(&e, "name") {
                        name = value.to_string();
                    }
                    continue;
                }
                b"ParameterValues" => parameters.push(
                    ParameterValues::from_xml(&mut reader, &e, step_id)
                        .unwrap()
                        .display()
                        .unwrap(),
                ),
                _ => {}
            },
            _ => {}
        }
        buf.clear()
    }

    let parameters = parameters.join(" ; ");

    if id_to_script_step(step_id) == ScriptStep::Comment {
        if parameters.trim().is_empty() {
            return Some("".to_string());
        } else {
            return Some(format!("# {parameters}"));
        }
    }

    let parameters = parameters.trim();
    if parameters.is_empty() {
        if matches!(
            id_to_script_step(step_id),
            ScriptStep::GoToField
                | ScriptStep::IfStart
                | ScriptStep::IfElse
                | ScriptStep::ExitLoopIf
        ) {
            return Some(format!("{name} []"));
        }

        return Some(name.to_string());
    };

    Some(format!("{name} [ {parameters} ]"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let xml = r#"<Step id="79" name="Fenster fixieren" enable="True">"#;

        let expected_output = Some("Fenster fixieren".to_string());
        let script_id: u32 = 79;
        assert_eq!(from_xml(&script_id, xml.trim()), expected_output);
    }

    #[test]
    fn test_enabled() {
        let xml = r#"
            <Step id="86" name="Fehleraufzeichnung setzen" enable="True">
                <Options>196608</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Boolean">
                        <Boolean id="131072" value="True"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Fehleraufzeichnung setzen [ ON ]".to_string());
        let script_id: u32 = 86;
        assert_eq!(from_xml(&script_id, xml.trim()), expected_output);
    }

    #[test]
    fn test_disabled() {
        let xml = r#"
            <Step id="86" name="Fehleraufzeichnung setzen" enable="True">
                <Options>196608</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Boolean">
                        <Boolean id="131072" value="False"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Fehleraufzeichnung setzen [ OFF ]".to_string());
        let script_id: u32 = 86;
        assert_eq!(from_xml(&script_id, xml.trim()), expected_output);
    }

    #[test]
    fn test_enter_find_mode() {
        let xml = r#"
            <Step id="22" name="Suchenmodus aktivieren" enable="True">
                <SourceUUID>3C5EBF9C-BAE0-460F-92AF-3FB73649951B</SourceUUID>
                <ParameterValues membercount="1">
                    <Parameter type="Boolean">
                        <Boolean type="Pause" id="16777216" value="False"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Suchenmodus aktivieren [ Pause: OFF ]".to_string());
        let script_id: u32 = 22;
        assert_eq!(from_xml(&script_id, xml.trim()), expected_output);
    }

    #[test]
    fn test_truncate_table_broken_reference() {
        let xml = r#"
            <Step index="509" id="182" name="Tabelle leeren" enable="True">
                <UUID>25ED2DB6-D1D0-402A-8B5A-6A06505C0A2A</UUID>
                <OwnerID></OwnerID>
                <Options>138</Options>
                <ParameterValues membercount="2">
                    <Parameter type="Boolean">
                        <Boolean type="Mit Dialog" id="128" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="List">
                        <List name="&lt;Tabelle nicht vorhanden&gt;" value="1"></List>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output =
            Some("Tabelle leeren [ Mit Dialog: OFF ; <Tabelle nicht vorhanden> ]".to_string());
        let script_id: u32 = 182;
        assert_eq!(from_xml(&script_id, xml.trim()), expected_output);
    }
}

#[cfg(test)]
mod commit_tests {
    use super::*;

    #[test]
    fn test() {
        let xml = r#"
            <Step id="75" name="Schreibe Änderung Datens./Abfrage" enable="True">
                <Options>384</Options>
                <ParameterValues membercount="3">
                    <Parameter type="Boolean">
                        <Boolean type="Dateneingabeüberprüfung unterdrücken" id="256" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Mit Dialog" id="128" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Schreiben erzwingen" id="512" value="False"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output =
            Some("Schreibe Änderung Datens./Abfrage [ Mit Dialog: OFF ]".to_string());
        let script_id: u32 = 75;
        assert_eq!(from_xml(&script_id, xml.trim()), expected_output);
    }

    #[test]
    fn test_force() {
        let xml = r#"
            <Step id="75" name="Schreibe Änderung Datens./Abfrage" enable="True">
                <Options>384</Options>
                <ParameterValues membercount="3">
                    <Parameter type="Boolean">
                        <Boolean type="Dateneingabeüberprüfung unterdrücken" id="256" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Mit Dialog" id="128" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Schreiben erzwingen" id="512" value="True"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(
            "Schreibe Änderung Datens./Abfrage [ Mit Dialog: OFF ; Schreiben erzwingen ]"
                .to_string(),
        );
        let script_id: u32 = 75;
        assert_eq!(from_xml(&script_id, xml.trim()), expected_output);
    }

    #[test]
    fn test_suppress_validate() {
        let xml = r#"
            <Step id="75" name="Schreibe Änderung Datens./Abfrage" enable="True">
                <Options>384</Options>
                <ParameterValues membercount="3">
                    <Parameter type="Boolean">
                        <Boolean type="Dateneingabeüberprüfung unterdrücken" id="256" value="True"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Mit Dialog" id="128" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Schreiben erzwingen" id="512" value="False"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Schreibe Änderung Datens./Abfrage [ Dateneingabeüberprüfung unterdrücken ; Mit Dialog: OFF ]".to_string());
        let script_id: u32 = 75;
        assert_eq!(from_xml(&script_id, xml.trim()), expected_output);
    }

    #[test]
    fn test_all_options() {
        let xml = r#"
            <Step id="75" name="Schreibe Änderung Datens./Abfrage" enable="True">
                <Options>384</Options>
                <ParameterValues membercount="3">
                    <Parameter type="Boolean">
                        <Boolean type="Dateneingabeüberprüfung unterdrücken" id="256" value="True"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Mit Dialog" id="128" value="True"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Schreiben erzwingen" id="512" value="True"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Schreibe Änderung Datens./Abfrage [ Dateneingabeüberprüfung unterdrücken ; Mit Dialog: ON ; Schreiben erzwingen ]".to_string());
        let script_id: u32 = 75;
        assert_eq!(from_xml(&script_id, xml.trim()), expected_output);
    }

    #[test]
    fn test_new_window() {
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
                                <DimParentWindow>False</DimParentWindow>
                            </Options>
                        </WindowReference>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let step_id: u32 = 122;
        let expected_output =
            Some("Neues Fenster [ Style: Dokument ; Layout: <Originallayout> ]".to_string());
        assert_eq!(from_xml(&step_id, xml.trim()), expected_output);
    }

    #[test]
    fn test_new_window_with_sizes() {
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

        let step_id: u32 = 122;
        let expected_output = Some(r#"Neues Fenster [ Style: Dokument ; Name: "Foo Bar" ; Layout: <Originallayout> ; Height: 100 ; Width: 200 ; Top: 300 ; Left: 400 ; Minimize: OFF ; Maximize: OFF ; Resize: OFF ; Menu: OFF ; Toolbar: OFF ]"#.to_string());
        assert_eq!(from_xml(&step_id, xml.trim()), expected_output);
    }
}
