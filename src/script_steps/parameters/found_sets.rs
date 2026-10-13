//! Stored find requests and sort orders.

use crate::script_steps::parameters::field_reference::FieldReference;
use crate::utils::xml_node::Node;

pub(crate) fn restore(node: &Node) -> Vec<String> {
    node.child("Restore")
        .filter(|restore| restore.attr_is_true("value"))
        .map(|_| "Restore".to_string())
        .into_iter()
        .collect()
}

/// Sort Records: `Sort order: Table::Field (Ascending) ; Keep records in sorted order`
pub(crate) fn sort_specification(node: &Node) -> Vec<String> {
    let Some(specification) = node.child("SortSpecification") else {
        return Vec::new();
    };
    let sorts: Vec<String> = specification
        .child("SortList")
        .into_iter()
        .flat_map(|list| list.children_named("Sort"))
        .map(|sort| {
            // A sort can also name a summary field; the sort field is the primary one
            let field =
                FieldReference::from_node(sort.child("PrimaryField").unwrap_or(sort)).display();
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
pub(crate) fn find_requests(node: &Node) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use crate::script_steps::parameters::parameter_values::parse;

    #[test]
    fn restore() {
        let on = r#"<Parameter type="Restore"><Restore value="True"></Restore></Parameter>"#;
        let off = r#"<Parameter type="Restore"><Restore value="False"></Restore></Parameter>"#;
        assert_eq!(parse(on, 0).as_deref(), Some("Restore"));
        assert_eq!(parse(off, 0), None);
    }

    #[test]
    fn sort_specification() {
        let xml = r#"<Parameter type="SortSpecification"><SortSpecification value="True" maintain="True"><SortList membercount="1"><Sort type="Descending"><PrimaryField><FieldReference id="1" name="Sort"><TableOccurrenceReference id="1" name="_Syntax"></TableOccurrenceReference></FieldReference></PrimaryField></Sort></SortList></SortSpecification></Parameter>"#;
        assert_eq!(
            parse(xml, 0).as_deref(),
            Some("Sort order: _Syntax::Sort (Descending) ; Keep records in sorted order")
        );
    }

    #[test]
    fn find_requests() {
        let xml = r#"<Parameter type="FindRequest"><FindRequestSet membercount="2"><FindRequest membercount="1" action="find"><find criteria="abc"><FieldReference id="15" name="Selector"><TableOccurrenceReference id="1" name="_Syntax"></TableOccurrenceReference></FieldReference></find></FindRequest><FindRequest membercount="0" action="omit"></FindRequest></FindRequestSet></Parameter>"#;
        assert_eq!(
            parse(xml, 0).as_deref(),
            Some(r#"Restore ; Find requests: Find (_Syntax::Selector: "abc"), Omit"#)
        );
    }
}
