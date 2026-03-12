use anyhow::{Error, Result, anyhow};
use std::path::PathBuf;

/// Validate that a path exists and is a directory
pub fn valid_dir_or_throw(dir_path: &PathBuf) -> Result<(), Error> {
    let metadata = std::fs::metadata(dir_path)
        .map_err(|_| anyhow!("Path '{}' not exists", dir_path.display()))?;

    if metadata.is_dir() {
        Ok(())
    } else {
        Err(anyhow!("Path '{}' is not a directory", dir_path.display()))
    }
}

/// Join scope ID and name into a formatted string
pub fn join_scope_id_and_name(scope_id: &str, scope_name: &str) -> String {
    format!("{scope_name} - ID {scope_id}")
}

/// Escape filename by replacing special characters with underscores
pub fn escape_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '{' => '_',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

pub fn should_skip_line(line: &str) -> bool {
    line.contains("<TagList></TagList>")
        || line.contains("<OwnerID></OwnerID>")
        || line.contains("<Options>0</Options>")
        || line.contains("<Options>1048576</Options>")
        || (line.contains("<UUID>") && line.contains("</UUID>"))
}
