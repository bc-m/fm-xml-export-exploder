use std::fs;
use std::path::Path;

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

pub fn should_skip_line(line: &str) -> bool {
    line.contains("<TagList></TagList>")
        || line.contains("<OwnerID></OwnerID>")
        || line.contains("<Options>0</Options>")
        || line.contains("<Options>1048576</Options>")
        || (line.contains("<UUID>") && line.contains("</UUID>"))
}

/// Recursively find all XML files in a directory and call `process_file` for each,
/// preserving the relative path structure from `base_dir` into `output_dir`.
pub fn for_each_xml_file(
    current_dir: &Path,
    base_dir: &Path,
    output_dir: &Path,
    process_file: &mut impl FnMut(&Path, &Path),
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("xml") {
            let relative_path = path.strip_prefix(base_dir).unwrap_or(&path);
            let output_file_path = output_dir.join(relative_path).with_extension("txt");
            if let Some(parent) = output_file_path.parent() {
                fs::create_dir_all(parent).unwrap_or_else(|err| {
                    panic!("Error creating directory {}: {}", parent.display(), err)
                });
            }
            process_file(&path, &output_file_path);
        } else if path.is_dir() {
            for_each_xml_file(&path, base_dir, output_dir, process_file);
        }
    }
}
