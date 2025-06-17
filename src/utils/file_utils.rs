use anyhow::{anyhow, Error, Result};
use std::path::PathBuf;

/// Validate that a path exists and is a directory
pub fn valid_dir_or_throw(dir_path: &PathBuf) -> Result<(), Error> {
    let metadata = std::fs::metadata(dir_path)
        .map_err(|_| anyhow!("Path '{}' not exists", dir_path.display()))?;

    match metadata.is_dir() {
        true => Ok(()),
        false => Err(anyhow!("Path '{}' is not a directory", dir_path.display())),
    }
}

/// Join scope ID and name into a formatted string
pub fn join_scope_id_and_name(scope_id: &str, scope_name: &str) -> String {
    format!("{} - ID {}", scope_name, scope_id)
}

/// Escape filename by replacing special characters with underscores
pub fn escape_filename(filename: &str) -> String {
    let escaped_filename = filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '{' => '_',
            _ => c,
        })
        .collect::<String>();

    escaped_filename.trim().to_string()
}

/// Check if a line should be skipped based on content
pub fn should_skip_line(line: &str) -> bool {
    if line.contains("<TagList></TagList>") {
        return true;
    };
    if line.contains("<OwnerID></OwnerID>") {
        return true;
    };
    if line.contains("<Options>0</Options>") {
        return true;
    }; // Default
    if line.contains("<Options>1048576</Options>") {
        return true;
    }; // Breakpoint
    if line.contains("<UUID>") && line.contains("</UUID>") {
        return true;
    };

    false
}
