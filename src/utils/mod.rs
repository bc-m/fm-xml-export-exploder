use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Error, Result};
use quick_xml::events::{BytesStart, Event};
use regex::Regex;

use crate::config::{CatalogType, Flags};
use crate::utils::attributes::get_attributes;
use crate::utils::file_utils::{escape_filename, join_scope_id_and_name, should_skip_line};
use crate::utils::xml_utils::{
    element_to_string, end_element_to_string, extract_values_from_xml_paths,
    start_element_to_string, text_element_to_string, XmlEventType,
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

/// Get the second to last element of a path
/// For example, "CustomFunctionCalc/CustomFunctionReference/@id" -> "CustomFunctionReference"
pub fn get_second_to_last(path: &str) -> Option<&str> {
    let parts: Vec<&str> = path.split('/').collect();
    parts.get(parts.len().saturating_sub(2)).copied()
}

impl Entity {
    pub fn _clear(&mut self) {
        self.id.clear();
        self.name.clear();
        self.tag_name.clear();
        self.element_with_id.clear();
        self.content.clear();
    }

    fn parse_xml_attributes(&mut self, e: &BytesStart) {
        for attr in get_attributes(e).unwrap() {
            match attr.0.as_str() {
                "id" => self.id = attr.1.to_string(),
                "name" => {
                    if self.name.is_empty() {
                        self.name = attr.1.to_string()
                    }
                }
                "Display" => self.name = attr.1.to_string(),
                _ => {}
            }
        }
    }

    pub fn read_xml_element<R: Read + BufRead>(
        &mut self,
        context: &mut ProcessingContext<'_, R>,
        start_tag: &BytesStart,
        _r_counter: usize,
        id_path: &str,
    ) {
        self.parse_xml_attributes(start_tag);
        self.tag_name = std::str::from_utf8(start_tag.name().as_ref())
            .unwrap_or("unknown")
            .to_string();

        if !self.id.is_empty() {
            let element_string = element_to_string(context, start_tag);
            self.content += element_string.as_str();
            if !id_path.is_empty() {
                let second_to_last = get_second_to_last(id_path).unwrap_or("unknown");
                if second_to_last == self.tag_name {
                    self.element_with_id = element_string;
                }
            }
            return;
        }

        self.content += start_element_to_string(start_tag, context.flags).as_str();
        let mut buf = Vec::new();
        loop {
            match context.reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    self.read_xml_element(context, &e, _r_counter + 1, id_path);
                }
                Ok(Event::Text(e)) => {
                    self.content += text_element_to_string(&e, false).as_str();
                }
                Ok(Event::End(e)) => {
                    self.content += end_element_to_string(&e).as_str();
                    break;
                }
                Ok(Event::Eof) => break,
                unknown_event => {
                    panic!("Wrong read event: {unknown_event:?}");
                }
            };
            buf.clear();
        }
    }
}

/// Represents the folder structure for catalog items that utilize folders, e.g. custom functions, scripts, layouts
#[derive(Debug, Clone)]
pub struct FolderStructure {
    /// Maps ID to the folder path where it should be placed
    pub item_paths: std::collections::HashMap<String, Vec<String>>,
}

impl FolderStructure {
    pub fn new() -> Self {
        Self {
            item_paths: std::collections::HashMap::new(),
        }
    }

    /// Get the folder path for a specific ID
    pub fn get_path_for_id(&self, id: &str) -> &[String] {
        self.item_paths.get(id).map(|v| v.as_slice()).unwrap_or(&[])
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
    entity.read_xml_element(context, start_tag, 1, id_path);
    if !entity.element_with_id.is_empty() {
        push_line_to_skeleton(
            context.skeleton,
            base_depth,
            1,
            entity.element_with_id.as_str(),
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
    let filename = join_scope_id_and_name(entity.id.as_str(), entity.name.as_str());
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
    let db_name = match &context.db_name {
        Some(db_name) => db_name,
        None => return Err(anyhow::anyhow!("Missing db name")),
    };
    let saxml_version = match &context.saxml_version {
        Some(saxml_version) => saxml_version,
        None => return Err(anyhow::anyhow!("Missing saxml version")),
    };

    let domain = match qualifier {
        Some(Qualifier::SanitizedScripts) => "script_sanitized".to_string(),
        _ => {
            match context.top_level_section {
                Some(TopLevelSection::Structure) => {
                    let mut domain_base = if let Some(catalog_type) = &context.catalog_type {
                        if catalog_type == &CatalogType::ValueList
                            && version_string_to_number(saxml_version)
                                < version_string_to_number("2.2.2.0")
                        {
                            "value_list_options".to_string() // Handle ValueList version-specific behavior
                        } else {
                            catalog_type.get_config().out_folder_name.clone()
                        }
                    } else {
                        "unknown_catalog".to_string()
                    };
                    // Add action suffix
                    if let Some(action) = &context.action {
                        domain_base.push_str(match action {
                            Action::Add => "",
                            Action::Modify => "__modify_action",
                            Action::Replace => "__replace_action",
                            Action::Delete => "__delete_action",
                        });
                    }
                    domain_base
                }
                _ => "_".to_string(),
            }
        }
    };

    let full_path = context
        .root_out_dir
        .clone()
        .join(match context.flags.output_tree {
            OutputTree::Db => PathBuf::from(db_name).join(domain),
            OutputTree::Domain => PathBuf::from(domain).join(db_name),
        });
    Ok(full_path)
}

/// Initialize (delete if exists, then create) output directory
pub fn _delete_then_create_dir(out_dir_path: &Path) {
    if out_dir_path.exists() {
        fs::remove_dir_all(out_dir_path).unwrap_or_else(|err| {
            panic!(
                "Error deleting directory {}: {}",
                out_dir_path.display(),
                err
            )
        });
    }
    fs::create_dir_all(out_dir_path).unwrap_or_else(|err| {
        panic!(
            "Error creating directory {}: {}",
            out_dir_path.display(),
            err
        )
    });
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
    let mut file_content = String::new();
    let reader = BufReader::new(content.as_bytes());
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        if !(flags.parse_all_lines || flags.lossless) && should_skip_line(&line) {
            continue;
        }
        file_content.push_str(
            line.strip_prefix(&"\t".repeat(remove_indent_count))
                .unwrap_or(&line),
        );
        file_content.push('\n');
    }

    write_file(output_file_path, &file_content);
}

pub fn write_text_file(output_file_path: &Path, content: &str) {
    let mut file_content = String::new();
    let regex = Regex::new(r"\r\n|\n\r|\r|\n").unwrap();
    for line in regex.split(content) {
        file_content.push_str(line);
        file_content.push('\n');
    }

    write_file(output_file_path, &file_content);
}

fn write_file(output_file_path: &Path, file_content: &String) {
    match File::create(output_file_path) {
        Ok(ref mut output_file) => {
            write!(output_file, "{file_content}").expect("Failed to write to file");
            output_file.flush().expect("Failed to flush output file");
        }
        Err(err) => {
            eprintln!(
                "Error creating file {}: {}",
                output_file_path.display(),
                err
            );
        }
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

    let indent =
        "\t".repeat(base_depth + relative_depth - 1 - if is_child_start_tag { 1 } else { 0 });
    let line = format!("{indent}{str_to_push}");

    if current_event_type == XmlEventType::Start || current_event_type == XmlEventType::Other {
        if skeleton.previous_event_type == XmlEventType::Start {
            skeleton.content.push_str(skeleton.previous_line.as_str());
            skeleton.content.push('\n');
        }
        skeleton.previous_line.clear();
        skeleton.previous_line.push_str(line.as_str());
    } else {
        let mut do_trim = current_event_type == XmlEventType::End
            && skeleton.previous_event_type != XmlEventType::End;
        if !skeleton.previous_line.is_empty() {
            skeleton.content.push_str(skeleton.previous_line.as_str());
            skeleton.previous_line.clear();
            if current_event_type == XmlEventType::End
                && skeleton.previous_event_type == XmlEventType::Other
            {
                skeleton.content.push('\n');
                do_trim = false;
            } else {
                do_trim = true;
            }
        }
        if do_trim {
            skeleton.content.push_str(line.as_str().trim());
        } else {
            skeleton.content.push_str(line.as_str());
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
    let parts: Vec<u64> = version
        .split('.')
        .map(|s| s.parse::<u64>().unwrap_or(0))
        .collect();

    let major = parts.first().copied().unwrap_or(0);
    let minor = parts.get(1).copied().unwrap_or(0);
    let patch = parts.get(2).copied().unwrap_or(0);
    let build = parts.get(3).copied().unwrap_or(0);

    major * 1_000_000_000 + minor * 1_000_000 + patch * 1_000 + build
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

// Some catalog items derive their name diferently from the standard method which builds the name using id and name attributes in the catalog item tag
pub fn rename_file_if_necessary(file_path: &Path, path_stack: &[Vec<u8>], tag_name: &[u8]) {
    // Check if path_stack has the required structure: second value should be "Structure" and third should be "AddAction"
    let has_structure_add_action = path_stack.len() >= 3
        && path_stack.get(1).is_some_and(|v| v == b"Structure")
        && path_stack.get(2).is_some_and(|v| v == b"AddAction");
    let has_structure_modify_action = path_stack.len() >= 3
        && path_stack.get(1).is_some_and(|v| v == b"Structure")
        && path_stack.get(2).is_some_and(|v| v == b"ModifyAction");

    let results = match tag_name {
        b"Account" if has_structure_add_action => {
            let paths = vec!["Account/Authentication/AccountName", "Account/@id"];
            Some(extract_values_from_xml_paths(file_path, &paths))
        }
        b"Authorization" if has_structure_add_action => {
            let paths = vec!["Authorization/Display", "Authorization/@id"];
            Some(extract_values_from_xml_paths(file_path, &paths))
        }
        b"BinaryData" if has_structure_add_action => {
            let paths = vec![
                "BinaryData/LibraryReference/@key",
                "BinaryData/LibraryReference/@id",
            ];
            Some(extract_values_from_xml_paths(file_path, &paths))
        }
        b"Layout" if has_structure_modify_action => {
            let paths = vec!["Layout/LayoutReference/@name", "Layout/LayoutReference/@id"];
            Some(extract_values_from_xml_paths(file_path, &paths))
        }
        b"Relationship" if has_structure_add_action => {
            let paths = vec![
                "Relationship/LeftTable/TableOccurrenceReference/@name",
                "Relationship/RightTable/TableOccurrenceReference/@name",
                "Relationship/@id",
            ];
            Some(extract_values_from_xml_paths(file_path, &paths))
        }
        _ => None,
    };

    if let Some(Ok(results)) = results {
        // Get the ID from the last element
        if let Some(Some(id)) = results.last() {
            // Join all names (except the last element which is the ID)
            let names: Vec<_> = results[..results.len() - 1]
                .iter()
                .filter_map(|r| r.as_ref())
                .collect();

            let name_part = if names.is_empty() {
                String::new()
            } else {
                format!(
                    "{} - ",
                    names
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(" - ")
                )
            };

            let new_name = format!("{name_part}ID {id}.xml");
            let _ = rename_file(file_path, &new_name);
        }
    }
}

/// Move a file to a subfolder if the subfolder path is not empty
pub fn move_to_subfolder(file_path: &Path, subfolder_dir_path: &Path) -> Result<PathBuf, String> {
    if subfolder_dir_path.to_string_lossy().is_empty() {
        return Ok(file_path.to_path_buf());
    }

    // Create the subfolder directory if it doesn't exist
    fs::create_dir_all(subfolder_dir_path)
        .map_err(|e| format!("Failed to create subfolder directory: {e}"))?;

    // Get the filename from the original path
    let filename = file_path
        .file_name()
        .ok_or_else(|| "File path has no filename".to_string())?;

    // Create the new path in the subfolder
    let new_path = subfolder_dir_path.join(filename);

    // Move the file to the new location
    fs::rename(file_path, &new_path)
        .map_err(|e| format!("Failed to move file to subfolder: {e}"))?;

    Ok(new_path)
}
