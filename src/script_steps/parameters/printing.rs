//! Print and Print Setup.

use crate::script_steps::parameters::format::labeled;
use crate::utils::xml_node::Node;

/// Print Setup: `Orientation: Portrait ; Paper size: 8.5" x 11"` (sizes are stored in 1/100 inch)
pub(crate) fn page_setup(node: &Node) -> Vec<String> {
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
pub(crate) fn print(node: &Node) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use crate::script_steps::parameters::parameter_values::parse;

    #[test]
    fn page_setup() {
        let xml = r#"<Parameter type="PageSetup"><PageSetup><Orientation name="Portrait" value="0"></Orientation><size height="1169.44" width="850"></size><scale value="100"></scale></PageSetup></Parameter>"#;
        assert_eq!(
            parse(xml, 0).as_deref(),
            Some(r#"Orientation: Portrait ; Paper size: 8.5" x 11.69""#)
        );
    }

    #[test]
    fn print() {
        let xml = r#"<Parameter type="Print"><Print name="Records being browsed" type="1" toFile="False"><Pages to="3" from="2" All="False"></Pages><Copies collate="True" value="4"></Copies></Print></Parameter>"#;
        assert_eq!(
            parse(xml, 0).as_deref(),
            Some("Records being browsed ; Pages: 2 to 3 ; Copies: 4")
        );
    }
}
