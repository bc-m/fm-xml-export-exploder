use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::{Result, bail};
use quick_xml::events::{BytesStart, Event};
use regex::Regex;

use crate::Skeleton;
use crate::config::{CatalogType, Flags, OutputTree};
use crate::utils::attributes::get_attributes;
use crate::utils::file_utils::{escape_filename, join_scope_id_and_name, should_skip_line};
use crate::utils::xml_utils::{
    XmlEventType, element_to_string, end_element_to_string, extract_values_from_xml_paths,
    general_ref_to_string, local_name_to_string, start_element_to_string, text_element_to_string,
    unescape_xml_entities,
};
use crate::xml_processor::{Action, ProcessingContext, Qualifier, TopLevelSection};

/// Version thresholds used for feature-gating catalog behavior.
pub(crate) const VERSION_2_2_2_0: u64 = version_string_to_number_const(2, 2, 2, 0);
pub(crate) const VERSION_2_2_3_4: u64 = version_string_to_number_const(2, 2, 3, 4);

pub(crate) mod attributes;
pub(crate) mod file_utils;
pub(crate) mod xml_utils;

pub fn rename_file(file_path: &Path, new_name: &str) -> Result<PathBuf> {
    let parent_dir = file_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("File path has no parent directory"))?;

    let new_path = parent_dir.join(new_name);

    fs::rename(file_path, &new_path)?;

    Ok(new_path)
}

#[derive(Debug, Default)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub tag_name: String,
    pub element_with_id: String,
    pub content: String,
}

impl Entity {
    fn parse_xml_attributes(&mut self, e: &BytesStart) {
        for attr in get_attributes(e) {
            match attr.0.as_str() {
                "id" => self.id = attr.1,
                "name" => {
                    if self.name.is_empty() {
                        self.name = unescape_xml_entities(attr.1)
                    }
                }
                "Display" => self.name = unescape_xml_entities(attr.1),
                _ => {}
            }
        }
    }

    pub fn read_xml_element<R: Read + BufRead>(
        &mut self,
        context: &mut ProcessingContext<'_, R>,
        start_tag: &BytesStart,
        id_path: &str,
    ) {
        self.parse_xml_attributes(start_tag);
        self.tag_name = local_name_to_string(start_tag.name().as_ref());

        if !self.id.is_empty() {
            let element_string = element_to_string(context, start_tag);
            self.content.push_str(&element_string);
            if !id_path.is_empty() {
                let parent_element = id_path.rsplit('/').nth(1).unwrap_or("unknown");
                if parent_element == self.tag_name {
                    self.element_with_id = element_string;
                }
            }
            return;
        }

        self.content
            .push_str(&start_element_to_string(start_tag, context.flags));
        let mut buf = Vec::new();
        loop {
            match context.reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    self.read_xml_element(context, &e, id_path);
                }
                Ok(Event::Text(e)) => {
                    self.content.push_str(&text_element_to_string(&e, false));
                }
                Ok(Event::GeneralRef(e)) => {
                    self.content.push_str(&general_ref_to_string(&e, true));
                }
                Ok(Event::End(e)) => {
                    self.content.push_str(&end_element_to_string(&e));
                    break;
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }
    }
}

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

pub fn write_rest_of_element_to_file<R: Read + BufRead>(
    context: &mut ProcessingContext<'_, R>,
    start_tag: &BytesStart,
    remove_indent_count: usize,
    base_depth: usize,
    id_path: &str,
) -> PathBuf {
    let mut entity = Entity::default();
    entity.read_xml_element(context, start_tag, id_path);
    if !entity.element_with_id.is_empty() {
        push_line_to_skeleton(
            context.skeleton,
            base_depth,
            1,
            &entity.element_with_id,
            false,
            XmlEventType::Other,
        );
    }
    write_entity_to_file(
        &context.current_out_dir,
        &entity,
        remove_indent_count,
        context.flags,
    )
}

pub fn write_entity_to_file(
    output_dir: &Path,
    entity: &Entity,
    remove_indent_count: usize,
    flags: &Flags,
) -> PathBuf {
    let filename = escape_filename(&join_scope_id_and_name(&entity.id, &entity.name));
    let output_file_path = output_dir.join(format!("{filename}.xml"));
    write_xml_file(
        &output_file_path,
        &entity.content,
        remove_indent_count,
        flags,
    );
    output_file_path
}

/// Build the output directory path by joining the domain and db_name in the correct order
/// depending on the output tree mode.
fn build_tree_path(root: &Path, output_tree: &OutputTree, domain: &str, db_name: &str) -> PathBuf {
    match output_tree {
        OutputTree::Db => root.join(db_name).join(domain),
        OutputTree::Domain => root.join(domain).join(db_name),
    }
}

/// Resolve the domain folder name for the current processing context.
fn resolve_structure_domain<R: Read + BufRead>(
    context: &ProcessingContext<'_, R>,
    ver: u64,
) -> String {
    let Some(TopLevelSection::Structure) = context.top_level_section else {
        return "_".to_string();
    };

    let folder_name = match context.catalog_type {
        Some(CatalogType::ValueList) if (VERSION_2_2_2_0..VERSION_2_2_3_4).contains(&ver) => {
            "value_list_stubs"
        }
        Some(CatalogType::CustomFunctions) if ver >= VERSION_2_2_3_4 => "custom_functions",
        Some(catalog_type) => catalog_type.get_config().out_folder_name,
        None => "unknown_catalog",
    };

    let action_suffix = match &context.action {
        Some(Action::Add) | None => "",
        Some(Action::Modify) => "__modify_action",
        Some(Action::Replace) => "__replace_action",
        Some(Action::Delete) => "__delete_action",
    };

    format!("{folder_name}{action_suffix}")
}

pub fn build_out_dir_path<R: Read + BufRead>(
    context: &ProcessingContext<'_, R>,
    qualifier: Option<Qualifier>,
) -> Result<PathBuf> {
    let Some(db_name) = &context.db_name else {
        bail!("Missing db name");
    };
    let Some(ver) = context.saxml_version_num else {
        bail!("Missing saxml version");
    };

    let domain = match qualifier {
        Some(Qualifier::SanitizedScripts) => "scripts_sanitized".to_string(),
        Some(Qualifier::SanitizedCustomFunctions) => "custom_functions_sanitized".to_string(),
        None => resolve_structure_domain(context, ver),
    };

    Ok(build_tree_path(
        context.root_out_dir,
        &context.flags.output_tree,
        &domain,
        db_name,
    ))
}

pub fn delete_output_directory(context: &ProcessingContext<'_, impl BufRead>) -> Result<()> {
    let Some(db_name) = &context.db_name else {
        bail!("Missing db name");
    };

    match context.flags.output_tree {
        OutputTree::Db => {
            // Delete ./db_name/
            let dir_to_delete = context.root_out_dir.join(db_name);
            if dir_to_delete.exists() {
                fs::remove_dir_all(&dir_to_delete)?;
            }
        }
        OutputTree::Domain => {
            // Delete all directories matching pattern ./*/db_name/
            if let Ok(entries) = fs::read_dir(context.root_out_dir) {
                for entry in entries.flatten() {
                    let target_dir = entry.path().join(db_name);
                    if target_dir.is_dir() {
                        fs::remove_dir_all(&target_dir)?;
                    }
                }
            }
        }
    }

    Ok(())
}

/// In Domain mode, migrate old-format `custom_functions/` output (pre-2.2.3.4 style)
/// where it contained `.txt` files directly instead of `.xml` files.
/// Renames the entire `custom_functions/` → `custom_functions_sanitized/` so git can track the rename.
pub fn migrate_old_custom_functions_if_needed(out_dir: &Path, flags: &Flags) -> Result<()> {
    if matches!(flags.output_tree, OutputTree::Db) {
        return Ok(());
    }

    let cf_dir = out_dir.join("custom_functions");
    let cf_sanitized_dir = out_dir.join("custom_functions_sanitized");

    // Only migrate if old dir exists and new dir does not
    if !cf_dir.exists() || cf_sanitized_dir.exists() {
        return Ok(());
    }

    // Check contents recursively: must have .txt files and NO .xml files
    let mut has_txt = false;
    let mut has_xml = false;
    fn check_dir(dir: &Path, has_txt: &mut bool, has_xml: &mut bool) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                check_dir(&path, has_txt, has_xml)?;
            } else if path.is_file() {
                match path.extension().and_then(|e| e.to_str()) {
                    Some("txt") => *has_txt = true,
                    Some("xml") => *has_xml = true,
                    _ => {}
                }
            }
        }
        Ok(())
    }
    check_dir(&cf_dir, &mut has_txt, &mut has_xml)?;

    if !has_txt || has_xml {
        return Ok(());
    }

    println!(
        "Migrating {} → {}",
        cf_dir.display(),
        cf_sanitized_dir.display()
    );
    fs::rename(&cf_dir, &cf_sanitized_dir)?;
    Ok(())
}

pub fn create_dir(dir_path: &Path) {
    fs::create_dir_all(dir_path)
        .unwrap_or_else(|err| panic!("Error creating directory {}: {}", dir_path.display(), err));
}

pub fn write_xml_file(
    output_file_path: &Path,
    content: &str,
    remove_indent_count: usize,
    flags: &Flags,
) {
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

    write_file(output_file_path, &file_content);
}

static LINE_SPLIT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\r\n|\n\r|\r|\n").unwrap());

pub fn write_text_file(output_file_path: &Path, content: &str) {
    let mut file_content = String::with_capacity(content.len());
    for line in LINE_SPLIT_REGEX.split(content) {
        file_content.push_str(line);
        file_content.push('\n');
    }

    write_file(output_file_path, &file_content);
}

fn write_file(output_file_path: &Path, file_content: &str) {
    let result = File::create(output_file_path).and_then(|mut f| {
        f.write_all(file_content.as_bytes())?;
        f.flush()
    });
    if let Err(err) = result {
        eprintln!("Error writing file {}: {}", output_file_path.display(), err);
    }
}

/// Push a line to the skeleton
/// But do this so that <tag>value</tag> is pushed as a single line instead of <tag>\nvalue\n</tag>
pub fn push_line_to_skeleton(
    skeleton: &mut Skeleton,
    base_depth: usize,
    relative_depth: usize,
    str_to_push: &str,
    is_child_start_tag: bool,
    current_event_type: XmlEventType,
) {
    if str_to_push.is_empty() {
        return;
    }

    let depth = base_depth + relative_depth - 1 - usize::from(is_child_start_tag);
    let indent = "\t".repeat(depth);
    let line = format!("{indent}{str_to_push}");

    if matches!(
        current_event_type,
        XmlEventType::Start | XmlEventType::Other
    ) {
        if skeleton.previous_event_type == XmlEventType::Start {
            skeleton.content.push_str(&skeleton.previous_line);
            skeleton.content.push('\n');
        }
        skeleton.previous_line.clear();
        skeleton.previous_line.push_str(&line);
    } else {
        // For non-start events (End, Text, CData, Comment), flush any pending line and
        // determine whether to inline (trim) the current content onto the same line.
        // This enables compact output like `<tag>value</tag>` on a single line.
        let do_trim = if skeleton.previous_line.is_empty() {
            current_event_type == XmlEventType::End
                && skeleton.previous_event_type != XmlEventType::End
        } else {
            skeleton.content.push_str(&skeleton.previous_line);
            skeleton.previous_line.clear();
            // After flushing a pending "Other" line followed by End, emit a newline instead of inlining
            let can_inline = !(current_event_type == XmlEventType::End
                && skeleton.previous_event_type == XmlEventType::Other);
            if !can_inline {
                skeleton.content.push('\n');
            }
            can_inline
        };
        if do_trim {
            skeleton.content.push_str(line.trim());
        } else {
            skeleton.content.push_str(&line);
        }
        if current_event_type == XmlEventType::End {
            skeleton.content.push('\n');
        }
    }
    skeleton.previous_event_type = current_event_type;
}

/// Convert version string to number for comparison
/// For example, "20.3.1.2" -> 20003001002
/// "21" -> 21000000000
pub fn version_string_to_number(version: &str) -> u64 {
    let multipliers = [1_000_000_000, 1_000_000, 1_000, 1];
    version
        .split('.')
        .zip(multipliers)
        .map(|(s, m)| s.parse::<u64>().unwrap_or(0) * m)
        .sum()
}

/// Const-evaluable version of `version_string_to_number` for compile-time constants.
const fn version_string_to_number_const(major: u64, minor: u64, patch: u64, build: u64) -> u64 {
    major * 1_000_000_000 + minor * 1_000_000 + patch * 1_000 + build
}

/// Some catalog items derive their name differently from the standard method
/// which builds the name using id and name attributes in the catalog item tag.
pub fn rename_file_if_necessary(file_path: &Path, path_stack: &[Vec<u8>], tag_name: &[u8]) {
    if path_stack.get(1).is_none_or(|v| v != b"Structure") {
        return;
    }
    let action = path_stack.get(2).map(|v| v.as_slice());

    let paths: &[&str] = match (action, tag_name) {
        (Some(b"AddAction"), b"Account") => &["Account/Authentication/AccountName", "Account/@id"],
        (Some(b"AddAction"), b"Authorization") => &["Authorization/Display", "Authorization/@id"],
        (Some(b"AddAction"), b"BinaryData") => &[
            "BinaryData/LibraryReference/@key",
            "BinaryData/LibraryReference/@id",
        ],
        (Some(b"ModifyAction"), b"Layout") => {
            &["Layout/LayoutReference/@name", "Layout/LayoutReference/@id"]
        }
        (Some(b"AddAction"), b"Relationship") => &[
            "Relationship/LeftTable/TableOccurrenceReference/@name",
            "Relationship/RightTable/TableOccurrenceReference/@name",
            "Relationship/@id",
        ],
        _ => return,
    };

    let Ok(results) = extract_values_from_xml_paths(file_path, paths) else {
        return;
    };
    let Some(id) = results.last().and_then(|r| r.as_deref()) else {
        return;
    };

    let name_parts: Vec<&str> = results[..results.len() - 1]
        .iter()
        .filter_map(|r| r.as_deref())
        .collect();

    let new_name = if name_parts.is_empty() {
        format!("ID {id}.xml")
    } else {
        format!("{} - ID {id}.xml", name_parts.join(" - "))
    };
    let _ = rename_file(file_path, &new_name);
}

/// Move a file to a subfolder if the subfolder path is not empty
pub fn move_to_subfolder(file_path: &Path, subfolder_dir_path: &Path) -> Result<PathBuf> {
    if subfolder_dir_path.as_os_str().is_empty() {
        return Ok(file_path.to_path_buf());
    }

    fs::create_dir_all(subfolder_dir_path)?;

    let filename = file_path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("File path has no filename"))?;

    let new_path = subfolder_dir_path.join(filename);

    fs::rename(file_path, &new_path)?;

    Ok(new_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_rename_file() {
        // Create a temporary file
        let temp_file = PathBuf::from("test_rename_source.txt");
        let content = "test content";
        fs::write(&temp_file, content).unwrap();

        // Verify the file exists
        assert!(temp_file.exists());

        // Rename the file
        let new_name = "test_rename_destination.txt";
        let result = rename_file(&temp_file, new_name);
        assert!(result.is_ok());

        let new_path = result.unwrap();
        assert_eq!(new_path.file_name().unwrap().to_str().unwrap(), new_name);
        assert!(new_path.exists());
        assert!(!temp_file.exists());

        // Verify content is preserved
        let read_content = fs::read_to_string(&new_path).unwrap();
        assert_eq!(read_content, content);

        // Clean up
        fs::remove_file(&new_path).unwrap();
    }

    #[test]
    fn test_rename_file_nonexistent() {
        let nonexistent_file = PathBuf::from("nonexistent_file.txt");
        let result = rename_file(&nonexistent_file, "new_name.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_file_if_necessary() {
        // Create a temporary XML file for testing
        let xml_content = r#"<Account id="1" kind="0" type="FileMaker" enable="False">
	<UUID modifications="1" userName="Mislav" accountName="Admin" timestamp="2025-04-16T18:22:20">28F730CD-2774-443F-AC9E-A46818A3C05F</UUID>
	<TagList></TagList>
	<Description></Description>
	<Authentication>
		<AccountName>[Guest]</AccountName>
		<PasswordEncrypted>BUI6dktVvYySpQ==</PasswordEncrypted>
	</Authentication>
	<PrivilegeSetReference id="3" name="[Read-Only Access]"></PrivilegeSetReference>
</Account>"#;

        let temp_file = PathBuf::from("test_account_rename.xml");
        std::fs::write(&temp_file, xml_content).unwrap();

        // Test with correct path_stack structure (should rename)
        let path_stack = vec![
            b"SomeElement".to_vec(),
            b"Structure".to_vec(),
            b"AddAction".to_vec(),
        ];
        rename_file_if_necessary(&temp_file, &path_stack, b"Account");

        // Check if file was renamed
        let renamed_file = PathBuf::from("[Guest] - ID 1.xml");
        assert!(renamed_file.exists());
        assert!(!temp_file.exists());

        // Clean up
        fs::remove_file(&renamed_file).unwrap();
    }

    #[test]
    fn test_rename_file_if_necessary_wrong_path_stack() {
        // Create a temporary XML file for testing
        let xml_content = r#"<Account id="1" kind="0" type="FileMaker" enable="False">
	<Authentication>
		<AccountName>[Guest]</AccountName>
	</Authentication>
</Account>"#;

        let temp_file = PathBuf::from("test_account_no_rename.xml");
        std::fs::write(&temp_file, xml_content).unwrap();

        // Test with wrong path_stack structure (should not rename)
        let path_stack = vec![
            b"SomeElement".to_vec(),
            b"WrongElement".to_vec(),
            b"AddAction".to_vec(),
        ];
        rename_file_if_necessary(&temp_file, &path_stack, b"Account");

        // Check that file was NOT renamed
        assert!(temp_file.exists());

        // Clean up
        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_move_to_subfolder() {
        // Create a temporary file
        let temp_file = PathBuf::from("test_move_source.txt");
        let content = "test content";
        fs::write(&temp_file, content).unwrap();

        // Verify the file exists
        assert!(temp_file.exists());

        // Create a subfolder
        let subfolder = PathBuf::from("test_subfolder");
        fs::create_dir(&subfolder).unwrap();

        // Move the file to subfolder
        let result = move_to_subfolder(&temp_file, &subfolder);
        assert!(result.is_ok());

        let new_path = result.unwrap();
        assert_eq!(
            new_path.file_name().unwrap().to_str().unwrap(),
            "test_move_source.txt"
        );
        assert!(new_path.exists());
        assert!(!temp_file.exists());

        // Verify content is preserved
        let read_content = fs::read_to_string(&new_path).unwrap();
        assert_eq!(read_content, content);

        // Clean up
        fs::remove_file(&new_path).unwrap();
        fs::remove_dir(&subfolder).unwrap();
    }

    #[test]
    fn test_move_to_subfolder_empty_path() {
        // Create a temporary file
        let temp_file = PathBuf::from("test_move_empty_source.txt");
        let content = "test content";
        fs::write(&temp_file, content).unwrap();

        // Verify the file exists
        assert!(temp_file.exists());

        // Try to move to empty path (should return original path)
        let empty_path = PathBuf::from("");
        let result = move_to_subfolder(&temp_file, &empty_path);
        assert!(result.is_ok());

        let returned_path = result.unwrap();
        assert_eq!(returned_path, temp_file);
        assert!(temp_file.exists());

        // Clean up
        fs::remove_file(&temp_file).unwrap();
    }
}
