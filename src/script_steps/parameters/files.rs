//! Paths and the Import/Export Records parameters.

use crate::script_steps::parameters::format::{labeled, quoted};
use crate::utils::xml_node::Node;

/// e.g. `"file:report.pdf" ; Automatically open ; Create email with file as attachment`
pub(crate) fn universal_path_list(node: &Node) -> Vec<String> {
    let Some(list) = node.child("UniversalPathList") else {
        return Vec::new();
    };
    let paths: Vec<&str> = list
        .descendant("ObjectList")
        .into_iter()
        .flat_map(|object_list| object_list.children_named("Location"))
        .map(|location| location.text())
        .collect();

    let mut parts = Vec::new();
    if !paths.is_empty() {
        parts.push(quoted(&paths.join("\n")));
    }
    if list.attr_is_true("AutoOpen") {
        parts.push("Automatically open".to_string());
    }
    if list.attr_is_true("CreateMail") {
        parts.push("Create email with file as attachment".to_string());
    }
    parts
}

/// Export Records: `Unicode (UTF-8) ; Apply current layout's data formatting`
pub(crate) fn export(node: &Node) -> Vec<String> {
    let Some(options) = node.child("Export").and_then(|e| e.child("Options")) else {
        return Vec::new();
    };
    let mut parts: Vec<String> = options
        .attr("name")
        .map(str::to_string)
        .into_iter()
        .collect();
    if options.attr_is_true("Formatting") {
        parts.push("Apply current layout's data formatting".to_string());
    }
    parts
}

/// Import Records: `Table: Contacts ; Update ; Add remaining data as new records ; Perform auto-enter options ; UTF-8`
pub(crate) fn import_field(node: &Node) -> Vec<String> {
    let Some(import) = node.child("ImportField") else {
        return Vec::new();
    };
    let mut parts = Vec::new();
    parts.extend(labeled(
        "Table",
        import
            .child("Target")
            .and_then(|t| t.child("TableOccurrenceReference"))
            .and_then(|t| t.attr("name"))
            .map(str::to_string),
    ));
    parts.extend(
        import
            .child("action")
            .and_then(|a| a.attr("name"))
            .map(str::to_string),
    );
    if import.attr_is_true("addasnew") {
        parts.push("Add remaining data as new records".to_string());
    }
    if import
        .child("Options")
        .is_some_and(|o| o.attr_is_true("doAutoEntry"))
    {
        parts.push("Perform auto-enter options".to_string());
    }
    parts.extend(
        import
            .child("CharacterSet")
            .and_then(|c| c.attr("name"))
            .map(str::to_string),
    );
    parts
}

#[cfg(test)]
mod tests {
    use crate::script_steps::parameters::parameter_values::parse;

    #[test]
    fn universal_path_list() {
        let xml = r#"<Parameter type="UniversalPathList"><UniversalPathList AutoOpen="True" CreateMail="True" membercount="2"><ObjectList><Location><![CDATA[file:a.txt]]></Location><Location><![CDATA[$Path]]></Location></ObjectList></UniversalPathList></Parameter>"#;
        assert_eq!(
            parse(xml, 0).as_deref(),
            Some(
                "\"file:a.txt\n$Path\" ; Automatically open ; Create email with file as attachment"
            )
        );
    }

    #[test]
    fn export() {
        let xml = r#"<Parameter type="Export"><Export><Options name="Unicode (UTF-8)" value="4" Formatting="True"></Options><Order type="Group"></Order></Export></Parameter>"#;
        assert_eq!(
            parse(xml, 0).as_deref(),
            Some("Unicode (UTF-8) ; Apply current layout's data formatting")
        );
    }

    #[test]
    fn import_field() {
        let xml = r#"<Parameter type="ImportField"><ImportField addasnew="True" isPredefined="False" delimiter="0" fieldNameRow="0"><Options copyContainersAsIs="False" matchFieldNames="False" doAutoEntry="True" splitRepetitions="False"></Options><Target><TableOccurrenceReference id="1" name="_Color"></TableOccurrenceReference></Target><Field membercount="1"><Map New="False" name="Source field 1 import to " id="0" kind="0" Options="0"><FieldReference id="0" name=""></FieldReference></Map></Field><CharacterSet name="UTF-8" value="5"></CharacterSet><action name="Update" value="2"></action></ImportField></Parameter>"#;
        assert_eq!(
            parse(xml, 0).as_deref(),
            Some(
                "Table: _Color ; Update ; Add remaining data as new records ; Perform auto-enter options ; UTF-8"
            )
        );
    }
}
