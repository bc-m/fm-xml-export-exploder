use std::fs;
use std::fs::File;
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::{Error, Result};
use quick_xml::events::{BytesStart, Event};
use regex::Regex;

use crate::config::{CatalogType, Flags};
use crate::utils::attributes::get_attributes;
use crate::utils::file_utils::{escape_filename, join_scope_id_and_name, should_skip_line};
use crate::utils::xml_utils::{
    XmlEventType, element_to_string, encode_xml_special_characters, end_element_to_string,
    extract_values_from_xml_paths, general_ref_to_string, local_name_to_string,
    start_element_to_string, text_element_to_string,
};
use crate::xml_processor::{Action, ProcessingContext, Qualifier, TopLevelSection};
use crate::{OutputTree, Skeleton};

pub(crate) mod attributes;
pub(crate) mod file_utils;
pub(crate) mod xml_utils;

pub fn rename_file(file_path: &Path, new_name: &str) -> Result<PathBuf, String> {
    let parent_dir = file_path
        .parent()
        .ok_or_else(|| "File path has no parent directory".to_string())?;

    let new_path = parent_dir.join(new_name);

    fs::rename(file_path, &new_path).map_err(|e| format!("Failed to rename file: {e}"))?;

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
                        self.name = encode_xml_special_characters(attr.1)
                    }
                }
                "Display" => self.name = encode_xml_special_characters(attr.1),
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
                let second_to_last = id_path.rsplit('/').nth(1).unwrap_or("unknown");
                if second_to_last == self.tag_name {
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
            };
            buf.clear();
        }
    }
}

/// Represents the folder structure for catalog items that utilize folders, e.g. custom functions, scripts, layouts
#[derive(Debug, Clone, Default)]
pub struct FolderStructure {
    /// Maps ID to the folder path where it should be placed
    pub item_paths: std::collections::HashMap<String, Vec<String>>,
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
    let filename = join_scope_id_and_name(&entity.id, &entity.name);
    let filename = escape_filename(&filename);

    let output_file_path = output_dir.join(format!("{filename}.xml"));
    write_xml_file(
        &output_file_path,
        &entity.content,
        remove_indent_count,
        flags,
    );
    output_file_path
}

pub fn build_out_dir_path<R: Read + BufRead>(
    context: &ProcessingContext<'_, R>,
    qualifier: Option<Qualifier>,
) -> Result<PathBuf, Error> {
    let Some(db_name) = &context.db_name else {
        return Err(anyhow::anyhow!("Missing db name"));
    };
    let Some(saxml_version) = &context.saxml_version else {
        return Err(anyhow::anyhow!("Missing saxml version"));
    };

    let domain = match qualifier {
        Some(Qualifier::SanitizedScripts) => "scripts_sanitized".to_string(),
        Some(Qualifier::SanitizedCustomFunctions) => "custom_functions_sanitized".to_string(),
        None => match context.top_level_section {
            Some(TopLevelSection::Structure) => {
                let ver = version_string_to_number(saxml_version);
                let folder_name = match context.catalog_type {
                    Some(CatalogType::ValueList)
                        if ver >= version_string_to_number("2.2.2.0")
                            && ver < version_string_to_number("2.2.3.4") =>
                    {
                        "value_list_stubs"
                    }
                    Some(CatalogType::CustomFunctions)
                        if ver >= version_string_to_number("2.2.3.4") =>
                    {
                        "custom_functions"
                    }
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
            _ => "_".to_string(),
        },
    };

    let full_path = context.root_out_dir.join(match context.flags.output_tree {
        OutputTree::Db => PathBuf::from(db_name).join(domain),
        OutputTree::Domain => PathBuf::from(domain).join(db_name),
    });
    Ok(full_path)
}

pub fn delete_output_directory(context: &ProcessingContext<'_, impl BufRead>) -> Result<(), Error> {
    let db_name = context.db_name.as_ref().unwrap();

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
            if let Ok(entries) = fs::read_dir(&context.root_out_dir) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        let target_dir = entry_path.join(db_name);
                        if target_dir.exists() {
                            fs::remove_dir_all(&target_dir)?;
                        }
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
pub fn migrate_old_custom_functions_if_needed(out_dir: &Path, flags: &Flags) -> Result<(), Error> {
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
    fn check_dir(dir: &Path, has_txt: &mut bool, has_xml: &mut bool) -> Result<(), Error> {
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
    let skip_noise = !flags.parse_all_lines && !flags.lossless;
    let mut file_content = String::new();
    for line in content.lines() {
        if skip_noise && should_skip_line(line) {
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
    let mut file_content = String::new();
    for line in LINE_SPLIT_REGEX.split(content) {
        file_content.push_str(line);
        file_content.push('\n');
    }

    write_file(output_file_path, &file_content);
}

fn write_file(output_file_path: &Path, file_content: &str) {
    let result = File::create(output_file_path).and_then(|mut f| {
        write!(f, "{file_content}")?;
        f.flush()
    });
    if let Err(err) = result {
        eprintln!(
            "Error writing file {}: {}",
            output_file_path.display(),
            err
        );
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
        let mut do_trim = current_event_type == XmlEventType::End
            && skeleton.previous_event_type != XmlEventType::End;
        if !skeleton.previous_line.is_empty() {
            skeleton.content.push_str(&skeleton.previous_line);
            skeleton.previous_line.clear();
            // After flushing a pending "Other" line followed by End, emit a newline instead of inlining
            do_trim = !(current_event_type == XmlEventType::End
                && skeleton.previous_event_type == XmlEventType::Other);
            if !do_trim {
                skeleton.content.push('\n');
            }
        }
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

/// Some catalog items derive their name differently from the standard method
/// which builds the name using id and name attributes in the catalog item tag.
pub fn rename_file_if_necessary(file_path: &Path, path_stack: &[Vec<u8>], tag_name: &[u8]) {
    let is_structure = path_stack.get(1).is_some_and(|v| v == b"Structure");
    if !is_structure {
        return;
    }
    let action = path_stack.get(2).map(|v| v.as_slice());
    let has_structure_add_action = action == Some(b"AddAction");
    let has_structure_modify_action = action == Some(b"ModifyAction");

    let paths: &[&str] = match tag_name {
        b"Account" if has_structure_add_action => {
            &["Account/Authentication/AccountName", "Account/@id"]
        }
        b"Authorization" if has_structure_add_action => {
            &["Authorization/Display", "Authorization/@id"]
        }
        b"BinaryData" if has_structure_add_action => &[
            "BinaryData/LibraryReference/@key",
            "BinaryData/LibraryReference/@id",
        ],
        b"Layout" if has_structure_modify_action => {
            &["Layout/LayoutReference/@name", "Layout/LayoutReference/@id"]
        }
        b"Relationship" if has_structure_add_action => &[
            "Relationship/LeftTable/TableOccurrenceReference/@name",
            "Relationship/RightTable/TableOccurrenceReference/@name",
            "Relationship/@id",
        ],
        _ => return,
    };

    let results = extract_values_from_xml_paths(file_path, paths);

    if let Ok(results) = results
        && let Some(Some(id)) = results.last()
    {
        let names: Vec<&str> = results[..results.len() - 1]
            .iter()
            .filter_map(|r| r.as_deref())
            .collect();

        let name_part = if names.is_empty() {
            String::new()
        } else {
            format!("{} - ", names.join(" - "))
        };

        let new_name = format!("{name_part}ID {id}.xml");
        let _ = rename_file(file_path, &new_name);
    }
}

/// Move a file to a subfolder if the subfolder path is not empty
pub fn move_to_subfolder(file_path: &Path, subfolder_dir_path: &Path) -> Result<PathBuf, String> {
    if subfolder_dir_path.as_os_str().is_empty() {
        return Ok(file_path.to_path_buf());
    }

    fs::create_dir_all(subfolder_dir_path)
        .map_err(|e| format!("Failed to create subfolder directory: {e}"))?;

    let filename = file_path
        .file_name()
        .ok_or_else(|| "File path has no filename".to_string())?;

    let new_path = subfolder_dir_path.join(filename);

    fs::rename(file_path, &new_path)
        .map_err(|e| format!("Failed to move file to subfolder: {e}"))?;

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
        assert!(result.unwrap_err().contains("Failed to rename file"));
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
