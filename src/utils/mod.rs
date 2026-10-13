use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::config::Flags;
use crate::utils::file_utils::should_skip_line;
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
pub fn derived_file_name(xml: &str, path_stack: &[String], tag_name: &str) -> Option<String> {
    if path_stack.get(1).is_none_or(|v| v != "Structure") {
        return None;
    }
    let action = path_stack.get(2).map(|v| v.as_str());

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

    Some(if name_parts.is_empty() {
        format!("ID {id}.xml")
    } else {
        format!("{} - ID {id}.xml", name_parts.join(" - "))
    })
}

pub fn create_dir(dir_path: &Path) {
    fs::create_dir_all(dir_path)
        .unwrap_or_else(|err| panic!("Error creating directory {}: {}", dir_path.display(), err));
}

/// Prepares exploded XML for writing: removes the given indentation and, unless all
/// lines are kept, noisy lines.
pub fn prepare_xml_content(content: &str, remove_indent_count: usize, flags: &Flags) -> String {
    let indent_prefix = "\t".repeat(remove_indent_count);
    let filter_noisy_lines = !flags.parse_all_lines && !flags.lossless;
    let mut file_content = String::with_capacity(content.len());
    for line in content.lines() {
        if filter_noisy_lines && should_skip_line(line) {
            continue;
        }
        file_content.push_str(line.strip_prefix(&indent_prefix).unwrap_or(line));
        file_content.push('\n');
    }
    file_content
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
    fn test_derived_file_name_wrong_path_stack() {
        assert_eq!(
            derived_file_name(ACCOUNT, &path_stack("WrongElement"), "Account"),
            None
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
