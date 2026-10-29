use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::config::Flags;
use crate::utils::file_utils::{escape_filename, join_scope_id_and_name, should_skip_line};
use crate::utils::output_files::OutputFiles;
use crate::utils::xml_utils::extract_values_from_xml_paths;

pub(crate) mod attributes;
pub(crate) mod entity;
pub(crate) mod file_utils;
pub(crate) mod output_files;
pub(crate) mod output_path;
pub(crate) mod skeleton;
pub(crate) mod xml_node;
pub(crate) mod xml_utils;

// Re-exports for backward compatibility
pub use entity::read_rest_of_element;
pub(crate) use output_path::VERSION_2_2_3_4;
pub use output_path::{
    build_out_dir_path, migrate_old_custom_functions_if_needed, output_roots,
    version_string_to_number,
};
pub use skeleton::push_line_to_skeleton;

/// Represents the folder structure for catalog items that utilize folders, e.g. custom functions, scripts, layouts
#[derive(Debug, Clone, Default)]
pub struct FolderStructure {
    /// Maps ID to the folder path where it should be placed
    pub item_paths: HashMap<String, Vec<String>>,
}

impl FolderStructure {
    /// Get the folder path for a specific ID
    pub fn get_path_for_id(&self, id: &str) -> &[String] {
        self.item_paths.get(id).map_or(&[], |v| v.as_slice())
    }
}

/// Some catalog items derive their file name from their content instead of the id and
/// name attributes of the catalog item tag, e.g. accounts are named after the account name.
/// The name may contain one folder, e.g. modified fields are grouped by their table.
pub fn derived_file_name(xml: &str, path_stack: &[String], tag_name: &str) -> Option<String> {
    if path_stack.get(1).is_none_or(|v| v != "Structure") {
        return None;
    }
    let action = path_stack.get(2).map(|v| v.as_str());

    // A modify action has one field catalog per field, so several of them share a table
    if (action, tag_name) == (Some("ModifyAction"), "FieldCatalog") {
        let results = extract_values_from_xml_paths(
            xml,
            &[
                "FieldCatalog/BaseTableReference/@name",
                "FieldCatalog/BaseTableReference/@id",
                "FieldCatalog/ObjectList/Field/FieldReference/@name",
                "FieldCatalog/ObjectList/Field/FieldReference/@id",
            ],
        )
        .ok()?;
        let [Some(table), Some(table_id), Some(field), Some(field_id)] = results.as_slice() else {
            return None;
        };
        return Some(format!(
            "{}/{}.xml",
            escape_filename(&join_scope_id_and_name(table_id, table)),
            escape_filename(&join_scope_id_and_name(field_id, field))
        ));
    }

    let paths: &[&str] = match (action, tag_name) {
        (Some("AddAction"), "Account") => &["Account/Authentication/AccountName", "Account/@id"],
        (Some("AddAction"), "Authorization") => &["Authorization/Display", "Authorization/@id"],
        (Some("AddAction"), "BinaryData") => &[
            "BinaryData/LibraryReference/@key",
            "BinaryData/LibraryReference/@id",
        ],
        (Some("ModifyAction"), "Layout") => {
            &["Layout/LayoutReference/@name", "Layout/LayoutReference/@id"]
        }
        (Some("AddAction"), "Relationship") => &[
            "Relationship/LeftTable/TableOccurrenceReference/@name",
            "Relationship/RightTable/TableOccurrenceReference/@name",
            "Relationship/@id",
        ],
        _ => return None,
    };

    let results = extract_values_from_xml_paths(xml, paths).ok()?;
    let id = results.last()?.as_deref()?;

    let name_parts: Vec<&str> = results[..results.len() - 1]
        .iter()
        .filter_map(|r| r.as_deref())
        .collect();

    Some(escape_filename(&if name_parts.is_empty() {
        format!("ID {id}.xml")
    } else {
        format!("{} - ID {id}.xml", name_parts.join(" - "))
    }))
}

pub fn create_dir(dir_path: &Path) {
    fs::create_dir_all(dir_path)
        .unwrap_or_else(|err| panic!("Error creating directory {}: {}", dir_path.display(), err));
}

/// Prepares exploded XML for writing: removes the given indentation and, unless all
/// lines are kept, noisy lines. Lines that continue a CDATA section or a text value
/// are content, so they keep their tabs, and carriage returns are kept as well.
pub fn prepare_xml_content(content: &str, remove_indent_count: usize, flags: &Flags) -> String {
    let indent_prefix = "\t".repeat(remove_indent_count);
    let filter_noisy_lines = !flags.parse_all_lines && !flags.lossless;
    let mut file_content = String::with_capacity(content.len());
    let mut state = LineState::Markup;
    for line in content.split_terminator('\n') {
        let is_markup = is_markup_line(line, state);
        state = state_after_line(line, state);
        if filter_noisy_lines && should_skip_line(line) {
            continue;
        }
        let line = if is_markup {
            line.strip_prefix(&indent_prefix).unwrap_or(line)
        } else {
            line
        };
        file_content.push_str(line);
        file_content.push('\n');
    }
    file_content
}

/// Where a line of XML starts or ends: between tags, inside a text value or inside a
/// CDATA section
#[derive(Clone, Copy, PartialEq)]
enum LineState {
    Markup,
    Text,
    CData,
}

/// Whether a line starting in `state` is indented markup rather than content
fn is_markup_line(line: &str, state: LineState) -> bool {
    let trimmed = line.trim_start();
    state == LineState::Markup && trimmed.starts_with('<') && !trimmed.starts_with("<![CDATA[")
}

/// The state at the end of a line that starts in `state`
fn state_after_line(line: &str, mut state: LineState) -> LineState {
    let mut rest = line;
    loop {
        if state == LineState::CData {
            let Some(end) = rest.find("]]>") else {
                return LineState::CData;
            };
            rest = &rest[end + 3..];
            state = LineState::Text;
            continue;
        }
        let Some(start) = rest.find('<') else {
            // Text without a tag on its own line continues a text value
            return if state == LineState::Text || !rest.trim().is_empty() {
                LineState::Text
            } else {
                LineState::Markup
            };
        };
        rest = &rest[start..];
        if let Some(cdata) = rest.strip_prefix("<![CDATA[") {
            rest = cdata;
            state = LineState::CData;
            continue;
        }
        let Some(end) = tag_end(rest) else {
            return LineState::Markup;
        };
        let is_start_tag = !rest.starts_with("</")
            && !rest.starts_with("<?")
            && !rest.starts_with("<!")
            && !rest[..end].ends_with('/');
        rest = &rest[end + 1..];
        // A start tag at the end of a line is followed by indented children
        if is_start_tag && rest.trim().is_empty() {
            return LineState::Markup;
        }
        state = if is_start_tag {
            LineState::Text
        } else {
            LineState::Markup
        };
    }
}

/// The index of the `>` that closes the tag at the start of `tag`, skipping quoted
/// attribute values
fn tag_end(tag: &str) -> Option<usize> {
    let mut quote = None;
    for (i, byte) in tag.bytes().enumerate() {
        match (quote, byte) {
            (None, b'"' | b'\'') => quote = Some(byte),
            (Some(q), _) if q == byte => quote = None,
            (None, b'>') => return Some(i),
            _ => {}
        }
    }
    None
}

pub fn write_xml_file(
    output: &OutputFiles,
    output_file_path: &Path,
    content: &str,
    remove_indent_count: usize,
    flags: &Flags,
) {
    output.write(
        output_file_path,
        &prepare_xml_content(content, remove_indent_count, flags),
    );
}

static LINE_SPLIT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\r\n|\n\r|\r|\n").unwrap());

pub fn write_text_file(output: &OutputFiles, output_file_path: &Path, content: &str) {
    let mut file_content = String::with_capacity(content.len());
    for line in LINE_SPLIT_REGEX.split(content) {
        file_content.push_str(line);
        file_content.push('\n');
    }

    output.write(output_file_path, &file_content);
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACCOUNT: &str = r#"<Account id="1" kind="0" type="FileMaker" enable="False">
	<UUID modifications="1" userName="Mislav" accountName="Admin" timestamp="2025-04-16T18:22:20">28F730CD-2774-443F-AC9E-A46818A3C05F</UUID>
	<TagList></TagList>
	<Description></Description>
	<Authentication>
		<AccountName>[Guest]</AccountName>
		<PasswordEncrypted>BUI6dktVvYySpQ==</PasswordEncrypted>
	</Authentication>
	<PrivilegeSetReference id="3" name="[Read-Only Access]"></PrivilegeSetReference>
</Account>"#;

    fn path_stack(section: &str) -> Vec<String> {
        vec![
            "SomeElement".to_owned(),
            section.to_owned(),
            "AddAction".to_owned(),
        ]
    }

    #[test]
    fn test_derived_file_name() {
        assert_eq!(
            derived_file_name(ACCOUNT, &path_stack("Structure"), "Account").as_deref(),
            Some("[Guest] - ID 1.xml")
        );
    }

    #[test]
    fn derived_file_names_are_escaped() {
        let account = ACCOUNT.replace("[Guest]", "Sales/Admin: \"Test\"");
        assert_eq!(
            derived_file_name(&account, &path_stack("Structure"), "Account").as_deref(),
            Some("Sales_Admin_ _Test_ - ID 1.xml")
        );
    }

    #[test]
    fn modified_fields_are_named_after_their_table_and_field() {
        let field_catalog = r#"<FieldCatalog>
	<BaseTableReference id="129" name="Test/Table"></BaseTableReference>
	<ObjectList membercount="1">
		<Field>
			<FieldReference id="12" name="Calc?" repetition="1"></FieldReference>
		</Field>
	</ObjectList>
</FieldCatalog>"#;
        let modify_action = vec![
            "FMSaveAsXML".to_owned(),
            "Structure".to_owned(),
            "ModifyAction".to_owned(),
        ];
        assert_eq!(
            derived_file_name(field_catalog, &modify_action, "FieldCatalog").as_deref(),
            Some("Test_Table - ID 129/Calc_ - ID 12.xml")
        );
        // Added fields keep one file per table
        assert_eq!(
            derived_file_name(field_catalog, &path_stack("Structure"), "FieldCatalog"),
            None
        );
    }

    #[test]
    fn test_derived_file_name_wrong_path_stack() {
        assert_eq!(
            derived_file_name(ACCOUNT, &path_stack("WrongElement"), "Account"),
            None
        );
    }

    fn lossless() -> Flags {
        Flags {
            parse_all_lines: false,
            lossless: true,
            output_tree: crate::config::OutputTree::Db,
            obfuscate_passwords: false,
            script_extension: "txt".to_string(),
        }
    }

    #[test]
    fn content_lines_keep_their_indentation() {
        let xml = "\t\t<Calculation>\n\t\t\t<Text><![CDATA[Let ( [\n\t\t\t\tx = 1\n] ; x )]]></Text>\n\t\t\t<Chunk>a\n\t\t\t\tb</Chunk>\n\t\t</Calculation>\n";
        assert_eq!(
            prepare_xml_content(xml, 2, &lossless()),
            "<Calculation>\n\t<Text><![CDATA[Let ( [\n\t\t\t\tx = 1\n] ; x )]]></Text>\n\t<Chunk>a\n\t\t\t\tb</Chunk>\n</Calculation>\n"
        );
    }

    #[test]
    fn carriage_returns_are_kept() {
        let xml = "\t<Text><![CDATA[Not (\r\n\tx\r\n)]]></Text>\n";
        assert_eq!(
            prepare_xml_content(xml, 1, &lossless()),
            "<Text><![CDATA[Not (\r\n\tx\r\n)]]></Text>\n"
        );
    }

    #[test]
    fn quoted_angle_brackets_do_not_end_a_tag() {
        let xml = "\t<Field name=\"a > b\">\n\t\t<Comment></Comment>\n\t</Field>\n";
        assert_eq!(
            prepare_xml_content(xml, 1, &lossless()),
            "<Field name=\"a > b\">\n\t<Comment></Comment>\n</Field>\n"
        );
    }

    #[test]
    fn test_derived_file_name_other_item() {
        assert_eq!(
            derived_file_name(ACCOUNT, &path_stack("Structure"), "Script"),
            None
        );
    }
}
