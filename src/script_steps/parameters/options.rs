//! The `Options` parameter of Pause/Resume Script, Insert File, Perform AppleScript and
//! Save Records as Excel/PDF.

use crate::script_steps::parameters::format::{labeled, on_off};
use crate::utils::xml_node::Node;

pub(crate) fn options(node: &Node) -> Vec<String> {
    let mut parts = Vec::new();
    for options in node.children_named("Options") {
        // Save Records as PDF: the type holds which records to save
        if options.child("Security").is_some() {
            parts.extend(pdf_options(options));
            continue;
        }
        match options.attr("type") {
            // Pause/Resume Script
            Some("Indefinitely") => parts.push("Indefinitely".to_string()),
            Some(label) if label.ends_with(": ") => {
                parts.extend(labeled(label.trim_end_matches(": "), options.calculation()));
            }
            // Perform AppleScript
            Some("Text") => parts.extend(
                Some(options.text().to_string())
                    .filter(|text| !text.is_empty())
                    .map(|text| format!("Native AppleScript: {text}")),
            ),
            Some("Calculation") => {
                parts.extend(labeled("Calculated AppleScript", options.calculation()));
            }
            // Insert File
            Some("Title") => parts.extend(labeled("Dialog title", options.calculation())),
            Some("Filters") => {
                let filters: Vec<String> = options
                    .descendant("Filters")
                    .into_iter()
                    .flat_map(|filters| filters.children_named("Filter"))
                    .map(|filter| {
                        let name = filter.calculation().unwrap_or_default();
                        match filter.attr("extension") {
                            Some(extension) => format!("{name} ({extension})"),
                            None => name,
                        }
                    })
                    .collect();
                if !filters.is_empty() {
                    parts.push(format!("Filters: {}", filters.join(", ")));
                }
            }
            Some(kind @ ("Storage" | "Display" | "Compress")) => {
                parts.extend(labeled(
                    kind,
                    options
                        .child(kind)
                        .and_then(|n| n.attr("name"))
                        .map(str::to_string),
                ));
            }
            // Save Records as Excel
            None => parts.extend(excel_options(options)),
            Some(_) => {}
        }
    }
    parts
}

/// Save Records as PDF. Passwords are only marked as required, never shown.
fn pdf_options(options: &Node) -> Vec<String> {
    let mut parts: Vec<String> = options
        .attr("type")
        .map(str::to_string)
        .into_iter()
        .collect();

    // Document: Title, Subject, Author, Keywords
    if let Some(document) = options.child("Document") {
        for parameter in document.children_named("Parameter") {
            if let Some(label) = parameter.attr("type") {
                parts.extend(labeled(label, parameter.calculation()));
            }
        }
    }

    if let Some(pages) = options.child("Pages") {
        let bound = |kind: &str| {
            pages
                .children_named("Parameter")
                .find(|p| p.attr("type") == Some(kind))
                .and_then(Node::calculation)
        };
        if pages
            .child("Include")
            .is_some_and(|i| i.attr_is_true("All"))
        {
            parts.push("All pages".to_string());
        } else {
            match (bound("from"), bound("to")) {
                (Some(from), Some(to)) => parts.push(format!("Pages: {from} to {to}")),
                (Some(from), None) => parts.push(format!("Pages: from {from}")),
                _ => {}
            }
        }
    }

    if let Some(security) = options.child("Security") {
        if security
            .child("Open")
            .is_some_and(|o| o.attr_is_true("value"))
        {
            parts.push("Require password to open document".to_string());
        }
        // Printing and editing restrictions only apply with a control password
        if security
            .child("Control")
            .is_some_and(|c| c.attr_is_true("value"))
        {
            parts.push("Require password to control printing and editing".to_string());
            for (child, label) in [("Print", "Printing"), ("Edit", "Editing")] {
                parts.extend(labeled(
                    label,
                    security
                        .child(child)
                        .and_then(|n| n.attr("name"))
                        .map(str::to_string),
                ));
            }
            for (child, label) in [
                ("EnableCopying", "Enable copying"),
                ("AllowScreenReader", "Enable screen reader"),
            ] {
                if let Some(node) = security.child(child) {
                    parts.push(format!("{label}: {}", on_off(node, "value")));
                }
            }
        }
    }

    if let Some(view) = options.child("View") {
        let initial_view: Vec<&str> = ["show", "Layout", "Magnification"]
            .into_iter()
            .filter_map(|child| view.child(child)?.attr("name"))
            .collect();
        if !initial_view.is_empty() {
            parts.push(format!("Initial view: {}", initial_view.join(", ")));
        }
    }
    parts
}

/// e.g. `Records being browsed ; Use field names as column names: ON ; Worksheet: $Sheet`
fn excel_options(options: &Node) -> Vec<String> {
    let mut parts = Vec::new();
    if let Some(save) = options.child("Save") {
        parts.extend(save.attr("type").map(str::to_string));
        if save.attr("useFieldNames").is_some() {
            parts.push(format!(
                "Use field names as column names: {}",
                on_off(save, "useFieldNames")
            ));
        }
    }
    for parameter in options.children_named("Parameter") {
        if let Some(label) = parameter.attr("type") {
            parts.extend(labeled(label, parameter.calculation()));
        }
    }
    parts
}

#[cfg(test)]
mod tests {
    use crate::script_steps::parameters::parameter_values::parse;

    #[test]
    fn pause_options() {
        let indefinitely =
            r#"<Parameter type="Options"><Options type="Indefinitely"></Options></Parameter>"#;
        assert_eq!(parse(indefinitely, 0).as_deref(), Some("Indefinitely"));

        let duration = r#"<Parameter type="Options"><Options type="Duration (seconds): "><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[1]]></Text></Calculation></Calculation></Options></Parameter>"#;
        assert_eq!(parse(duration, 0).as_deref(), Some("Duration (seconds): 1"));
    }

    #[test]
    fn insert_file_options() {
        let xml = r#"<Parameter type="Options"><Options type="Title"><Title><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[$Title]]></Text></Calculation></Calculation></Title></Options><Options type="Filters"><Filters size="1"><Filter extension="*"><Calculation><Text><![CDATA["All Files"]]></Text></Calculation></Filter></Filters></Options><Options type="Storage"><Storage name="Let user choose" value="0"></Storage></Options><Options type="Display"><Display name="Display icon" value="0"></Display></Options><Options type="Compress"><Compress name="Never compress" value="1"></Compress></Options></Parameter>"#;
        assert_eq!(
            parse(xml, 0).as_deref(),
            Some(
                r#"Dialog title: $Title ; Filters: "All Files" (*) ; Storage: Let user choose ; Display: Display icon ; Compress: Never compress"#
            )
        );
    }

    #[test]
    fn excel_options() {
        let xml = r#"<Parameter type="Options"><Options><Save type="Records being browsed" value="1" useFieldNames="True"></Save><Parameter type="Worksheet"><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[$WKS1]]></Text></Calculation></Calculation></Parameter><Parameter type="Title"><Calculation datatype="1" position="1"><Calculation><Text><![CDATA[$Title]]></Text></Calculation></Calculation></Parameter></Options></Parameter>"#;
        assert_eq!(
            parse(xml, 0).as_deref(),
            Some(
                "Records being browsed ; Use field names as column names: ON ; Worksheet: $WKS1 ; Title: $Title"
            )
        );
    }

    #[test]
    fn pdf_options_defaults() {
        let xml = r#"<Parameter type="Options"><Options type="Current record" value="2"><Document></Document><Pages><Parameter type="from"><Calculation datatype="1" position="4"><Calculation><Text><![CDATA[1]]></Text></Calculation></Calculation></Parameter><Include All="True"></Include></Pages><Security><Open value="False"></Open><Control value="False"></Control><Print name="High Resolution" value="2"></Print><Edit name="Any except extracting pages" value="4"></Edit><EnableCopying value="True"></EnableCopying><AllowScreenReader value="True"></AllowScreenReader></Security><View><show name="Pages Panel and Page" value="2"></show><Layout name="Single Page" value="1"></Layout><Magnification name="100%" value="7"></Magnification></View></Options></Parameter>"#;
        assert_eq!(
            parse(xml, 0).as_deref(),
            Some(
                "Current record ; All pages ; Initial view: Pages Panel and Page, Single Page, 100%"
            )
        );
    }

    #[test]
    fn pdf_options_secured() {
        let calc = |text: &str| {
            format!(
                r#"<Calculation datatype="1" position="0"><Calculation><Text><![CDATA[{text}]]></Text></Calculation></Calculation>"#
            )
        };
        let xml = format!(
            r#"<Parameter type="Options"><Options type="Records being browsed" value="1"><Document><Parameter type="Title">{}</Parameter><Parameter type="Keywords">{}</Parameter></Document><Pages><Parameter type="from">{}</Parameter><Parameter type="to">{}</Parameter><Include All="False"></Include></Pages><Security><Open value="True"><Parameter type="Password">{}</Parameter></Open><Control value="True"><Parameter type="Password">{}</Parameter></Control><Print name="Low Resolution" value="1"></Print><Edit name="Inserting, deleting and rotating pages" value="1"></Edit><EnableCopying value="False"></EnableCopying><AllowScreenReader value="True"></AllowScreenReader></Security><View><show name="Page Only" value="0"></show><Layout name="Single Page" value="1"></Layout><Magnification name="800%" value="12"></Magnification></View></Options></Parameter>"#,
            calc("$Title"),
            calc("$Keywords"),
            calc("2"),
            calc("5"),
            calc("$OpenPassword"),
            calc("$EditPassword"),
        );
        let display = parse(&xml, 0).unwrap();
        assert_eq!(
            display,
            "Records being browsed ; Title: $Title ; Keywords: $Keywords ; Pages: 2 to 5 ; Require password to open document ; Require password to control printing and editing ; Printing: Low Resolution ; Editing: Inserting, deleting and rotating pages ; Enable copying: OFF ; Enable screen reader: ON ; Initial view: Page Only, Single Page, 800%"
        );
        assert!(!display.contains("Password"));
    }

    #[test]
    fn applescript_options() {
        let native = r#"<Parameter type="Options"><Options type="Text">display dialog &quot;Hi&quot;</Options></Parameter>"#;
        assert_eq!(
            parse(native, 0).as_deref(),
            Some(r#"Native AppleScript: display dialog "Hi""#)
        );

        let calculated = r#"<Parameter type="Options"><Options type="Calculation"><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[$script]]></Text></Calculation></Calculation></Options></Parameter>"#;
        assert_eq!(
            parse(calculated, 0).as_deref(),
            Some("Calculated AppleScript: $script")
        );

        let empty = r#"<Parameter type="Options"><Options type="Text"></Options></Parameter>"#;
        assert_eq!(parse(empty, 0), None);
    }

    #[test]
    fn empty_options() {
        let xml = r#"<Parameter type="Options"><Options></Options></Parameter>"#;
        assert_eq!(parse(xml, 0), None);
    }
}
