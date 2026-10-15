use std::fs;
use std::path::Path;

use crate::utils::output_files::OutputFiles;
use crate::utils::write_text_file;

use anyhow::{Result, anyhow, ensure};

/// Validate that a path exists and is a directory
pub fn valid_dir_or_throw(dir_path: &Path) -> Result<()> {
    let metadata = fs::metadata(dir_path)
        .map_err(|_| anyhow!("Path '{}' does not exist", dir_path.display()))?;
    ensure!(
        metadata.is_dir(),
        "Path '{}' is not a directory",
        dir_path.display()
    );
    Ok(())
}

/// Join scope ID and name into a formatted string
pub fn join_scope_id_and_name(scope_id: &str, scope_name: &str) -> String {
    format!("{scope_name} - ID {scope_id}")
}

/// Escape filename by replacing special characters with underscores
pub fn escape_filename(filename: &str) -> String {
    filename
        .trim()
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '{' => '_',
            _ => c,
        })
        .collect()
}

const SKIP_PATTERNS: &[&str] = &[
    "<TagList></TagList>",
    "<OwnerID></OwnerID>",
    "<Options>0</Options>",
    "<Options>1048576</Options>",
];

pub fn should_skip_line(line: &str) -> bool {
    SKIP_PATTERNS.iter().any(|p| line.contains(p))
        || (line.contains("<UUID>") && line.contains("</UUID>"))
}

/// Writes the text version of an exploded file with the given extension, mirroring its
/// path below `xml_dir` into `text_dir`. No text file is written when `text` is `None`.
pub fn write_text_version(
    xml_path: &Path,
    xml_dir: &Path,
    text_dir: &Path,
    extension: &str,
    output: &OutputFiles,
    text: Option<String>,
) {
    let relative_path = xml_path.strip_prefix(xml_dir).unwrap_or(xml_path);
    let text_path = text_dir.join(relative_path).with_extension(extension);
    if let Some(parent) = text_path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|err| panic!("Error creating directory {}: {}", parent.display(), err));
    }
    if let Some(text) = text {
        write_text_file(output, &text_path, &text);
    }
}
