use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;

use crate::config::Flags;
use crate::utils::file_utils::should_skip_line;
use crate::utils::xml_utils::extract_values_from_xml_paths;

pub(crate) mod attributes;
pub(crate) mod entity;
pub(crate) mod file_utils;
pub(crate) mod output_path;
pub(crate) mod skeleton;
pub(crate) mod xml_utils;

// Re-exports for backward compatibility
pub use entity::write_rest_of_element_to_file;
pub(crate) use output_path::VERSION_2_2_3_4;
pub use output_path::{
    build_out_dir_path, delete_output_directory, migrate_old_custom_functions_if_needed,
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

pub fn rename_file(file_path: &Path, new_name: &str) -> Result<PathBuf> {
    let parent_dir = file_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("File path has no parent directory"))?;

    let new_path = parent_dir.join(new_name);

    fs::rename(file_path, &new_path)?;

    Ok(new_path)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_rename_file() {
        let temp_file = PathBuf::from("test_rename_source.txt");
        let content = "test content";
        fs::write(&temp_file, content).unwrap();

        assert!(temp_file.exists());

        let new_name = "test_rename_destination.txt";
        let result = rename_file(&temp_file, new_name);
        assert!(result.is_ok());

        let new_path = result.unwrap();
        assert_eq!(new_path.file_name().unwrap().to_str().unwrap(), new_name);
        assert!(new_path.exists());
        assert!(!temp_file.exists());

        let read_content = fs::read_to_string(&new_path).unwrap();
        assert_eq!(read_content, content);

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

        let path_stack = vec![
            b"SomeElement".to_vec(),
            b"Structure".to_vec(),
            b"AddAction".to_vec(),
        ];
        rename_file_if_necessary(&temp_file, &path_stack, b"Account");

        let renamed_file = PathBuf::from("[Guest] - ID 1.xml");
        assert!(renamed_file.exists());
        assert!(!temp_file.exists());

        fs::remove_file(&renamed_file).unwrap();
    }

    #[test]
    fn test_rename_file_if_necessary_wrong_path_stack() {
        let xml_content = r#"<Account id="1" kind="0" type="FileMaker" enable="False">
	<Authentication>
		<AccountName>[Guest]</AccountName>
	</Authentication>
</Account>"#;

        let temp_file = PathBuf::from("test_account_no_rename.xml");
        std::fs::write(&temp_file, xml_content).unwrap();

        let path_stack = vec![
            b"SomeElement".to_vec(),
            b"WrongElement".to_vec(),
            b"AddAction".to_vec(),
        ];
        rename_file_if_necessary(&temp_file, &path_stack, b"Account");

        assert!(temp_file.exists());

        fs::remove_file(&temp_file).unwrap();
    }

    #[test]
    fn test_move_to_subfolder() {
        let temp_file = PathBuf::from("test_move_source.txt");
        let content = "test content";
        fs::write(&temp_file, content).unwrap();

        assert!(temp_file.exists());

        let subfolder = PathBuf::from("test_subfolder");
        fs::create_dir(&subfolder).unwrap();

        let result = move_to_subfolder(&temp_file, &subfolder);
        assert!(result.is_ok());

        let new_path = result.unwrap();
        assert_eq!(
            new_path.file_name().unwrap().to_str().unwrap(),
            "test_move_source.txt"
        );
        assert!(new_path.exists());
        assert!(!temp_file.exists());

        let read_content = fs::read_to_string(&new_path).unwrap();
        assert_eq!(read_content, content);

        fs::remove_file(&new_path).unwrap();
        fs::remove_dir(&subfolder).unwrap();
    }

    #[test]
    fn test_move_to_subfolder_empty_path() {
        let temp_file = PathBuf::from("test_move_empty_source.txt");
        let content = "test content";
        fs::write(&temp_file, content).unwrap();

        assert!(temp_file.exists());

        let empty_path = PathBuf::from("");
        let result = move_to_subfolder(&temp_file, &empty_path);
        assert!(result.is_ok());

        let returned_path = result.unwrap();
        assert_eq!(returned_path, temp_file);
        assert!(temp_file.exists());

        fs::remove_file(&temp_file).unwrap();
    }
}
