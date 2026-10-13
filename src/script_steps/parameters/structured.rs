use quick_xml::Reader;
use quick_xml::events::BytesStart;

use crate::script_steps::parameters::boolean::Boolean;
use crate::script_steps::parameters::field_reference::FieldReference;
use crate::script_steps::parameters::node::Node;

/// Parameters with nested structures, parsed into a [`Node`] tree first.
pub fn is_structured(parameter_type: &str) -> bool {
    matches!(
        parameter_type,
        "UniversalPathList"
            | "Options"
            | "Restore"
            | "Select"
            | "Object"
            | "Source"
            | "CustomMenuSet"
            | "Parameter"
            | "URL"
            | "PageSetup"
            | "Print"
            | "SQL"
            | "SortSpecification"
            | "FindRequest"
            | "Email"
            | "Variable"
            | "Flow"
            | "Export"
            | "ImportField"
    )
}

pub fn from_xml(
    reader: &mut Reader<&[u8]>,
    e: &BytesStart,
    parameter_type: &str,
) -> Option<String> {
    let node = Node::from_xml(reader, e);
    let parts = match parameter_type {
        "UniversalPathList" => universal_path_list(&node),
        "Options" => options(&node),
        "Restore" => restore(&node),
        "Select" => select(&node),
        "Object" => object(&node),
        "Source" => source(&node),
        "CustomMenuSet" => custom_menu_set(&node),
        "Parameter" => parameter(&node),
        "URL" => url(&node),
        "PageSetup" => page_setup(&node),
        "Print" => print(&node),
        "SQL" => sql(&node),
        "SortSpecification" => sort_specification(&node),
        "FindRequest" => find_requests(&node),
        "Email" => email(&node),
        "Variable" => variable(&node),
        "Flow" => labeled(
            "Flow",
            node.child("Flow")
                .and_then(|f| f.attr("value"))
                .map(str::to_string),
        )
        .into_iter()
        .collect(),
        "Export" => export(&node),
        "ImportField" => import_field(&node),
        _ => Vec::new(),
    };
    Some(parts.join(" ; ")).filter(|display| !display.is_empty())
}

fn on_off(node: &Node, attribute: &str) -> &'static str {
    Boolean::on_off(node.attr_is_true(attribute))
}

/// Text that FileMaker shows inside quotes, e.g. file paths.
fn quoted(text: &str) -> String {
    format!("\"{text}\"")
}

fn labeled(label: &str, value: Option<String>) -> Option<String> {
    value.map(|value| format!("{label}: {value}"))
}

/// e.g. `"file:report.pdf" ; Automatically open ; Create email with file as attachment`
fn universal_path_list(node: &Node) -> Vec<String> {
    let Some(list) = node.child("UniversalPathList") else {
        return Vec::new();
    };
    let paths: Vec<&str> = list
        .descendant("ObjectList")
        .into_iter()
        .flat_map(|object_list| object_list.children_named("Location"))
        .map(|location| location.text.as_str())
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

fn options(node: &Node) -> Vec<String> {
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

fn restore(node: &Node) -> Vec<String> {
    node.child("Restore")
        .filter(|restore| restore.attr_is_true("value"))
        .map(|_| "Restore".to_string())
        .into_iter()
        .collect()
}

/// Set Selection: `Start Position: 1 ; End Position: 12`
fn select(node: &Node) -> Vec<String> {
    [("Start", "Start Position"), ("End", "End Position")]
        .into_iter()
        .filter_map(|(child, label)| labeled(label, node.child(child)?.calculation()))
        .collect()
}

/// `Object Name: "name" ; Repetition: 1`
fn object(node: &Node) -> Vec<String> {
    [("Name", "Object Name"), ("repetition", "Repetition")]
        .into_iter()
        .filter_map(|(child, label)| labeled(label, node.child(child)?.calculation()))
        .collect()
}

/// AVPlayer Play: a layout object, a field or a URL
fn source(node: &Node) -> Vec<String> {
    let Some(source) = node.child("Source") else {
        return Vec::new();
    };
    match source.attr("type") {
        Some("LayoutObject") => object(source),
        Some("FieldReference") => {
            let mut parts = vec![format!(
                "Field: {}",
                FieldReference::from_node(source).display()
            )];
            parts.extend(labeled(
                "Repetition",
                source.descendant("repetition").and_then(Node::calculation),
            ));
            parts
        }
        Some("URL") => labeled("URL", source.calculation()).into_iter().collect(),
        _ => Vec::new(),
    }
}

/// Install Menu Set: `"[Standard FileMaker Menus]" ; Use as file default: OFF`
fn custom_menu_set(node: &Node) -> Vec<String> {
    let Some(reference) = node.child("CustomMenuSetReference") else {
        return Vec::new();
    };
    let mut parts: Vec<String> = reference.attr("name").map(quoted).into_iter().collect();
    for boolean in reference.children_named("Boolean") {
        if let Some(label) = boolean.attr("type") {
            parts.push(format!("{label}: {}", on_off(boolean, "value")));
        }
    }
    parts
}

/// Perform Script on Server wraps its calculation in a nested `<Parameter>`,
/// Perform JavaScript in Web Viewer lists one calculation per function parameter.
fn parameter(node: &Node) -> Vec<String> {
    let calculations: Vec<String> = match node.child("Parameter") {
        Some(nested) => nested.calculation().into_iter().collect(),
        None => node
            .children_named("Calculation")
            .filter_map(Node::calculation)
            .collect(),
    };
    match calculations.len() {
        0 => Vec::new(),
        1 => vec![format!("Parameter: {}", calculations[0])],
        _ => vec![format!("Parameters: {}", calculations.join(" ; "))],
    }
}

/// `URL: $url ; Automatically encode URL: ON`
fn url(node: &Node) -> Vec<String> {
    let Some(url) = node.child("URL") else {
        return Vec::new();
    };
    let Some(calculation) = url.calculation() else {
        return Vec::new();
    };
    vec![
        format!("URL: {calculation}"),
        format!("Automatically encode URL: {}", on_off(url, "autoEncode")),
    ]
}

/// Print Setup: `Orientation: Portrait ; Paper size: 8.5" x 11"` (sizes are stored in 1/100 inch)
fn page_setup(node: &Node) -> Vec<String> {
    let Some(page_setup) = node.child("PageSetup") else {
        return Vec::new();
    };
    let mut parts = Vec::new();
    parts.extend(labeled(
        "Orientation",
        page_setup
            .child("Orientation")
            .and_then(|o| o.attr("name"))
            .map(str::to_string),
    ));
    if let Some(size) = page_setup.child("size")
        && let (Some(width), Some(height)) =
            (inches(size.attr("width")), inches(size.attr("height")))
    {
        parts.push(format!("Paper size: {width}\" x {height}\""));
    }
    parts
}

fn inches(hundredths: Option<&str>) -> Option<String> {
    let inches = hundredths?.parse::<f64>().ok()? / 100.0;
    let formatted = format!("{inches:.2}");
    Some(
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string(),
    )
}

/// Print: `Records being browsed ; Pages: 2 to 3 ; Copies: 4`
fn print(node: &Node) -> Vec<String> {
    let Some(print) = node.child("Print") else {
        return Vec::new();
    };
    let mut parts: Vec<String> = print.attr("name").map(str::to_string).into_iter().collect();
    if let Some(pages) = print.child("Pages") {
        if pages.attr_is_true("All") {
            parts.push("All pages".to_string());
        } else if let (Some(from), Some(to)) = (pages.attr("from"), pages.attr("to")) {
            parts.push(format!("Pages: {from} to {to}"));
        }
    }
    if let Some(copies) = print.child("Copies").and_then(|c| c.attr("value"))
        && copies != "1"
    {
        parts.push(format!("Copies: {copies}"));
    }
    if print.attr_is_true("toFile") {
        parts.push("Print to file".to_string());
    }
    parts
}

/// Execute SQL: `ODBC: dsn ; SQL Text: SELECT ...`
fn sql(node: &Node) -> Vec<String> {
    let Some(sql) = node.child("SQL") else {
        return Vec::new();
    };
    let mut parts = Vec::new();
    if let Some(dsn) = sql.child("DSN").and_then(|d| d.attr("name"))
        && !dsn.is_empty()
    {
        parts.push(format!("ODBC: {dsn}"));
    }
    parts.extend(labeled("SQL Text", sql.calculation()));
    parts
}

/// Sort Records: `Sort order: Table::Field (Ascending) ; Keep records in sorted order`
fn sort_specification(node: &Node) -> Vec<String> {
    let Some(specification) = node.child("SortSpecification") else {
        return Vec::new();
    };
    let sorts: Vec<String> = specification
        .child("SortList")
        .into_iter()
        .flat_map(|list| list.children_named("Sort"))
        .map(|sort| {
            let field = FieldReference::from_node(sort).display();
            match sort.attr("type") {
                Some(order) => format!("{field} ({order})"),
                None => field,
            }
        })
        .collect();

    let mut parts = Vec::new();
    if !sorts.is_empty() {
        parts.push(format!("Sort order: {}", sorts.join(", ")));
    }
    if specification.attr_is_true("maintain") {
        parts.push("Keep records in sorted order".to_string());
    }
    parts
}

/// Enter Find Mode / Perform Find: `Restore ; Find requests: Find (Table::Field: "criteria"), Omit`
fn find_requests(node: &Node) -> Vec<String> {
    let requests: Vec<String> = node
        .descendant("FindRequestSet")
        .into_iter()
        .flat_map(|set| set.children_named("FindRequest"))
        .map(|request| {
            let action = match request.attr("action") {
                Some("omit") => "Omit",
                _ => "Find",
            };
            let criteria: Vec<String> = request
                .children_named("find")
                .map(|find| {
                    let field = FieldReference::from_node(find).display();
                    let criteria = find.attr("criteria").unwrap_or_default();
                    format!("{field}: \"{criteria}\"")
                })
                .collect();
            if criteria.is_empty() {
                action.to_string()
            } else {
                format!("{action} ({})", criteria.join(", "))
            }
        })
        .collect();

    if requests.is_empty() {
        Vec::new()
    } else {
        vec![
            "Restore".to_string(),
            format!("Find requests: {}", requests.join(", ")),
        ]
    }
}

/// Send Mail. Passwords and private keys are left out on purpose.
fn email(node: &Node) -> Vec<String> {
    let mut parts = Vec::new();
    for boolean in node.children_named("Boolean") {
        if let Some(label) = boolean.attr("type") {
            parts.push(format!("{label}: {}", on_off(boolean, "value")));
        }
    }

    let send = node.child("Send");
    let via = node
        .child("SMTP")
        .map(|smtp| ("Send via SMTP Server", smtp))
        .or_else(|| {
            node.child("OAuthAuthentication")
                .map(|oauth| ("Send via OAuth", oauth))
        });
    match via {
        Some((label, account)) => {
            parts.push(label.to_string());
            for child in &account.children {
                let label = match child.name.as_str() {
                    "Name" => "From name",
                    "Email" => "From address",
                    "ReplyTo" => "Reply-to",
                    "Server" => "SMTP server",
                    "Port" => "Port",
                    "UserName" => "User name",
                    "UserID" => "User ID",
                    "ServiceAccount" => "Service account",
                    "Encryption" | "Authentication" | "OAuthProvider" => {
                        let label = match child.name.as_str() {
                            "OAuthProvider" => "OAuth provider",
                            name => name,
                        };
                        parts.extend(labeled(label, child.attr("name").map(str::to_string)));
                        continue;
                    }
                    _ => continue,
                };
                parts.extend(labeled(label, child.calculation()));
            }
        }
        None if send.is_some() => parts.push("Send via E-mail Client".to_string()),
        None => {}
    }

    if let Some(send) = send {
        if send
            .child("Multiple")
            .is_some_and(|m| m.attr_is_true("value"))
        {
            parts.push("Multiple emails (one for each record in found set)".to_string());
        }
        for (child, label) in [
            ("To", "To"),
            ("CC", "CC"),
            ("BCC", "BCC"),
            ("Subject", "Subject"),
            ("Message", "Message"),
        ] {
            parts.extend(labeled(
                label,
                send.child(child).and_then(Node::calculation),
            ));
        }
    }

    let attachments = universal_path_list(node);
    if let Some(paths) = attachments.first() {
        parts.push(format!("Attachments: {paths}"));
    }
    parts
}

/// Target variable, e.g. `Target: $path` or `Target: $path[2]`
fn variable(node: &Node) -> Vec<String> {
    let Some(name) = node
        .child("Name")
        .and_then(|n| n.attr("value"))
        .filter(|n| !n.is_empty())
    else {
        return Vec::new();
    };
    let repetition = node.child("repetition").and_then(Node::calculation);
    match repetition.as_deref() {
        Some(rep) if rep != "1" => vec![format!("Target: {name}[{rep}]")],
        _ => vec![format!("Target: {name}")],
    }
}

/// Export Records: `Unicode (UTF-8) ; Apply current layout's data formatting`
fn export(node: &Node) -> Vec<String> {
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
fn import_field(node: &Node) -> Vec<String> {
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
    use quick_xml::Reader;
    use quick_xml::events::Event;

    use super::*;

    fn parse(xml: &str) -> Option<String> {
        let mut reader = Reader::from_str(xml);
        let Ok(Event::Start(e)) = reader.read_event() else {
            panic!("Wrong read event");
        };
        let parameter_type =
            crate::utils::attributes::parse_unescaped_attribute(&e, "type").unwrap();
        from_xml(&mut reader, &e, &parameter_type)
    }

    #[test]
    fn universal_path_list() {
        let xml = r#"<Parameter type="UniversalPathList"><UniversalPathList AutoOpen="True" CreateMail="True" membercount="2"><ObjectList><Location><![CDATA[file:a.txt]]></Location><Location><![CDATA[$Path]]></Location></ObjectList></UniversalPathList></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some(
                "\"file:a.txt\n$Path\" ; Automatically open ; Create email with file as attachment"
            )
        );
    }

    #[test]
    fn pause_options() {
        let indefinitely =
            r#"<Parameter type="Options"><Options type="Indefinitely"></Options></Parameter>"#;
        assert_eq!(parse(indefinitely).as_deref(), Some("Indefinitely"));

        let duration = r#"<Parameter type="Options"><Options type="Duration (seconds): "><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[1]]></Text></Calculation></Calculation></Options></Parameter>"#;
        assert_eq!(parse(duration).as_deref(), Some("Duration (seconds): 1"));
    }

    #[test]
    fn insert_file_options() {
        let xml = r#"<Parameter type="Options"><Options type="Title"><Title><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[$Title]]></Text></Calculation></Calculation></Title></Options><Options type="Filters"><Filters size="1"><Filter extension="*"><Calculation><Text><![CDATA["All Files"]]></Text></Calculation></Filter></Filters></Options><Options type="Storage"><Storage name="Let user choose" value="0"></Storage></Options><Options type="Display"><Display name="Display icon" value="0"></Display></Options><Options type="Compress"><Compress name="Never compress" value="1"></Compress></Options></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some(
                r#"Dialog title: $Title ; Filters: "All Files" (*) ; Storage: Let user choose ; Display: Display icon ; Compress: Never compress"#
            )
        );
    }

    #[test]
    fn excel_options() {
        let xml = r#"<Parameter type="Options"><Options><Save type="Records being browsed" value="1" useFieldNames="True"></Save><Parameter type="Worksheet"><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[$WKS1]]></Text></Calculation></Calculation></Parameter><Parameter type="Title"><Calculation datatype="1" position="1"><Calculation><Text><![CDATA[$Title]]></Text></Calculation></Calculation></Parameter></Options></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some(
                "Records being browsed ; Use field names as column names: ON ; Worksheet: $WKS1 ; Title: $Title"
            )
        );
    }

    #[test]
    fn pdf_options_defaults() {
        let xml = r#"<Parameter type="Options"><Options type="Current record" value="2"><Document></Document><Pages><Parameter type="from"><Calculation datatype="1" position="4"><Calculation><Text><![CDATA[1]]></Text></Calculation></Calculation></Parameter><Include All="True"></Include></Pages><Security><Open value="False"></Open><Control value="False"></Control><Print name="High Resolution" value="2"></Print><Edit name="Any except extracting pages" value="4"></Edit><EnableCopying value="True"></EnableCopying><AllowScreenReader value="True"></AllowScreenReader></Security><View><show name="Pages Panel and Page" value="2"></show><Layout name="Single Page" value="1"></Layout><Magnification name="100%" value="7"></Magnification></View></Options></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
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
        let display = parse(&xml).unwrap();
        assert_eq!(
            display,
            "Records being browsed ; Title: $Title ; Keywords: $Keywords ; Pages: 2 to 5 ; Require password to open document ; Require password to control printing and editing ; Printing: Low Resolution ; Editing: Inserting, deleting and rotating pages ; Enable copying: OFF ; Enable screen reader: ON ; Initial view: Page Only, Single Page, 800%"
        );
        assert!(!display.contains("Password"));
    }

    #[test]
    fn empty_options() {
        let xml = r#"<Parameter type="Options"><Options></Options></Parameter>"#;
        assert_eq!(parse(xml), None);
    }

    #[test]
    fn restore() {
        let on = r#"<Parameter type="Restore"><Restore value="True"></Restore></Parameter>"#;
        let off = r#"<Parameter type="Restore"><Restore value="False"></Restore></Parameter>"#;
        assert_eq!(parse(on).as_deref(), Some("Restore"));
        assert_eq!(parse(off), None);
    }

    #[test]
    fn select() {
        let xml = r#"<Parameter type="Select"><Start><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[1]]></Text></Calculation></Calculation></Start><End><Calculation datatype="1" position="1"><Calculation><Text><![CDATA[12]]></Text></Calculation></Calculation></End></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some("Start Position: 1 ; End Position: 12")
        );
    }

    #[test]
    fn object() {
        let xml = r#"<Parameter type="Object"><Name><Calculation datatype="1" position="0"><Calculation><Text><![CDATA["RepField"]]></Text></Calculation></Calculation></Name><repetition><Calculation datatype="1" position="1"><Calculation><Text><![CDATA[$Rep]]></Text></Calculation></Calculation></repetition></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some(r#"Object Name: "RepField" ; Repetition: $Rep"#)
        );
    }

    #[test]
    fn source() {
        let url = r#"<Parameter type="Source"><Source type="URL"><Calculation datatype="1" position="5"><Calculation><Text><![CDATA["http://example.com"]]></Text></Calculation></Calculation></Source></Parameter>"#;
        assert_eq!(parse(url).as_deref(), Some(r#"URL: "http://example.com""#));

        let field = r#"<Parameter type="Source"><Source type="FieldReference"><FieldReference id="1" name="Video"><TableOccurrenceReference id="1" name="Media"></TableOccurrenceReference><repetition><Calculation datatype="1" position="10"><Calculation><Text><![CDATA[2]]></Text></Calculation></Calculation></repetition></FieldReference></Source></Parameter>"#;
        assert_eq!(
            parse(field).as_deref(),
            Some("Field: Media::Video ; Repetition: 2")
        );
    }

    #[test]
    fn custom_menu_set() {
        let xml = r#"<Parameter type="CustomMenuSet"><CustomMenuSetReference id="1" name="[Standard FileMaker Menus]"><Boolean type="Use as file default" id="512" value="False"></Boolean></CustomMenuSetReference></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some(r#""[Standard FileMaker Menus]" ; Use as file default: OFF"#)
        );
    }

    #[test]
    fn parameter() {
        let nested = r#"<Parameter type="Parameter"><Parameter><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[$p]]></Text></Calculation></Calculation></Parameter></Parameter>"#;
        assert_eq!(parse(nested).as_deref(), Some("Parameter: $p"));

        let empty = r#"<Parameter type="Parameter"><Parameter></Parameter></Parameter>"#;
        assert_eq!(parse(empty), None);

        let multiple = r#"<Parameter type="Parameter"><Calculation datatype="1" position="10"><Calculation><Text><![CDATA[$p1]]></Text></Calculation></Calculation><Calculation datatype="1" position="11"><Calculation><Text><![CDATA[$p2]]></Text></Calculation></Calculation></Parameter>"#;
        assert_eq!(parse(multiple).as_deref(), Some("Parameters: $p1 ; $p2"));
    }

    #[test]
    fn page_setup() {
        let xml = r#"<Parameter type="PageSetup"><PageSetup><Orientation name="Portrait" value="0"></Orientation><size height="1169.44" width="850"></size><scale value="100"></scale></PageSetup></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some(r#"Orientation: Portrait ; Paper size: 8.5" x 11.69""#)
        );
    }

    #[test]
    fn print() {
        let xml = r#"<Parameter type="Print"><Print name="Records being browsed" type="1" toFile="False"><Pages to="3" from="2" All="False"></Pages><Copies collate="True" value="4"></Copies></Print></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some("Records being browsed ; Pages: 2 to 3 ; Copies: 4")
        );
    }

    #[test]
    fn sql() {
        let xml = r#"<Parameter type="SQL"><SQL suppress="False" DSN="False"><DSN name="" UserName="" Password=""></DSN><Calculation><Text><![CDATA[SELECT * FROM orders]]></Text></Calculation></SQL></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some("SQL Text: SELECT * FROM orders")
        );
    }

    #[test]
    fn sort_specification() {
        let xml = r#"<Parameter type="SortSpecification"><SortSpecification value="True" maintain="True"><SortList membercount="1"><Sort type="Descending"><PrimaryField><FieldReference id="1" name="Sort"><TableOccurrenceReference id="1" name="_Syntax"></TableOccurrenceReference></FieldReference></PrimaryField></Sort></SortList></SortSpecification></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some("Sort order: _Syntax::Sort (Descending) ; Keep records in sorted order")
        );
    }

    #[test]
    fn find_requests() {
        let xml = r#"<Parameter type="FindRequest"><FindRequestSet membercount="2"><FindRequest membercount="1" action="find"><find criteria="abc"><FieldReference id="15" name="Selector"><TableOccurrenceReference id="1" name="_Syntax"></TableOccurrenceReference></FieldReference></find></FindRequest><FindRequest membercount="0" action="omit"></FindRequest></FindRequestSet></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some(r#"Restore ; Find requests: Find (_Syntax::Selector: "abc"), Omit"#)
        );
    }

    #[test]
    fn email_smtp() {
        let calc = |text: &str| {
            format!(
                r#"<Calculation datatype="1" position="0"><Calculation><Text><![CDATA[{text}]]></Text></Calculation></Calculation>"#
            )
        };
        let xml = format!(
            r#"<Parameter type="Email"><Boolean type="With dialog" value="False"></Boolean><SMTP><Name>{}</Name><Server>{}</Server><Encryption name="TLS" value="2"></Encryption><Password>{}</Password></SMTP><Send OAuthAuthentication="False" SMTP="True"><Multiple value="False"></Multiple><To><CollectAddresses value="False"></CollectAddresses>{}</To><Subject>{}</Subject></Send><UniversalPathList membercount="1"><ObjectList><Location><![CDATA[$file]]></Location></ObjectList></UniversalPathList></Parameter>"#,
            calc("$name"),
            calc("$smtp"),
            calc("$secret"),
            calc("$to"),
            calc("$subject"),
        );
        assert_eq!(
            parse(&xml).as_deref(),
            Some(
                r#"With dialog: OFF ; Send via SMTP Server ; From name: $name ; SMTP server: $smtp ; Encryption: TLS ; To: $to ; Subject: $subject ; Attachments: "$file""#
            )
        );
    }

    #[test]
    fn variable() {
        let xml = r#"<Parameter type="Variable"><value></value><Name value="$var"></Name><repetition><Calculation datatype="1" position="2"><Calculation><Text><![CDATA[2]]></Text></Calculation></Calculation></repetition></Parameter>"#;
        assert_eq!(parse(xml).as_deref(), Some("Target: $var[2]"));
    }

    #[test]
    fn export() {
        let xml = r#"<Parameter type="Export"><Export><Options name="Unicode (UTF-8)" value="4" Formatting="True"></Options><Order type="Group"></Order></Export></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some("Unicode (UTF-8) ; Apply current layout's data formatting")
        );
    }

    #[test]
    fn import_field() {
        let xml = r#"<Parameter type="ImportField"><ImportField addasnew="True" isPredefined="False" delimiter="0" fieldNameRow="0"><Options copyContainersAsIs="False" matchFieldNames="False" doAutoEntry="True" splitRepetitions="False"></Options><Target><TableOccurrenceReference id="1" name="_Color"></TableOccurrenceReference></Target><Field membercount="1"><Map New="False" name="Source field 1 import to " id="0" kind="0" Options="0"><FieldReference id="0" name=""></FieldReference></Map></Field><CharacterSet name="UTF-8" value="5"></CharacterSet><action name="Update" value="2"></action></ImportField></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some(
                "Table: _Color ; Update ; Add remaining data as new records ; Perform auto-enter options ; UTF-8"
            )
        );
    }

    #[test]
    fn url() {
        let xml = r#"<Parameter type="URL"><URL autoEncode="False"><Calculation datatype="1" position="0"><Calculation><Text><![CDATA[$url]]></Text></Calculation></Calculation></URL></Parameter>"#;
        assert_eq!(
            parse(xml).as_deref(),
            Some("URL: $url ; Automatically encode URL: OFF")
        );
    }
}
