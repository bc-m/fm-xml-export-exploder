//! Combines command line options, the optional config file and the defaults into the
//! settings of a run. Command line options win over the config file, which wins over
//! the defaults.

use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use serde::Deserialize;

use crate::config::{Flags, OutputTree};

/// Name of the config file that is used automatically when it is in the current directory
pub const CONFIG_FILE_NAME: &str = "fm-xml-export-exploder.toml";

/// Parse all as XML exported FileMaker solutions from source directory and explode them to target directory.
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The source directory to read input (or `source` in the config file)
    source: Option<PathBuf>,

    /// The target directory to write output (or `target` in the config file)
    target: Option<PathBuf>,

    /// Config file with default options [default: ./fm-xml-export-exploder.toml if present]
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Parse all lines (or skip less important ones to reduce noise)
    #[arg(short, long)]
    all_lines: bool,

    /// Retain all information from the main xml (or skip less important catalogs and attributes)
    #[arg(short, long)]
    lossless: bool,

    /// Specify the output tree root folder: domain or db (default)
    #[arg(short = 't', long = "output-tree", alias = "output_tree", value_enum)]
    output_tree: Option<OutputTree>,

    /// File extension of the readable scripts in scripts_sanitized, e.g. fmscript for
    /// syntax highlighting in an editor (with or without a leading dot) [default: txt]
    #[arg(long, value_parser = parse_file_extension)]
    script_extension: Option<String>,

    /// Replace password values in sanitized output with bullet placeholders (••••••••)
    #[arg(long)]
    obfuscate_passwords: bool,

    /// Commit the output with git: one repository per database folder (-t db) or per
    /// domain folder (-t domain). Folders inside another git repository are skipped.
    #[arg(long)]
    git: bool,
}

/// The options of a config file; all of them are optional.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    source: Option<PathBuf>,
    target: Option<PathBuf>,
    output_tree: Option<OutputTree>,
    git: Option<bool>,
    script_extension: Option<String>,
    obfuscate_passwords: Option<bool>,
    all_lines: Option<bool>,
    lossless: Option<bool>,
}

impl ConfigFile {
    /// Parses a config file; relative paths are resolved against `base_dir`.
    pub fn parse(content: &str, base_dir: &Path) -> Result<ConfigFile> {
        let mut config: ConfigFile = toml::from_str(content)?;
        config.source = config.source.map(|path| resolve_path(base_dir, &path));
        config.target = config.target.map(|path| resolve_path(base_dir, &path));
        config.script_extension = config
            .script_extension
            .map(|extension| parse_file_extension(&extension))
            .transpose()
            .map_err(anyhow::Error::msg)?;
        Ok(config)
    }

    fn load(path: &Path) -> Result<ConfigFile> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Cannot read config file {}", path.display()))?;
        let path = fs::canonicalize(path)?;
        let base_dir = path.parent().unwrap_or(Path::new("/"));
        ConfigFile::parse(&content, base_dir)
            .with_context(|| format!("Invalid config file {}", path.display()))
    }
}

/// What a run does, after combining command line, config file and defaults
pub struct Settings {
    pub source: PathBuf,
    pub target: PathBuf,
    pub git: bool,
    pub flags: Flags,
    /// The config file that was used, if any
    pub config_path: Option<PathBuf>,
}

impl Settings {
    /// Uses the config file given with `--config`, or the one in `current_dir` if present.
    pub fn from_args(args: Args, current_dir: &Path) -> Result<Settings> {
        let config_path = match &args.config {
            Some(path) => Some(path.clone()),
            None => Some(current_dir.join(CONFIG_FILE_NAME)).filter(|path| path.is_file()),
        };
        let config = config_path
            .as_deref()
            .map(ConfigFile::load)
            .transpose()?
            .unwrap_or_default();
        Settings::combine(args, config, config_path)
    }

    fn combine(args: Args, config: ConfigFile, config_path: Option<PathBuf>) -> Result<Settings> {
        let Some(source) = args.source.or(config.source) else {
            bail!(
                "Missing source directory: pass it as argument or set `source` in the config file"
            );
        };
        let Some(target) = args.target.or(config.target) else {
            bail!(
                "Missing target directory: pass it as argument or set `target` in the config file"
            );
        };
        // A switch on the command line turns an option on; otherwise the config decides
        let switch = |cli: bool, config: Option<bool>| cli || config.unwrap_or(false);
        Ok(Settings {
            source,
            target,
            git: switch(args.git, config.git),
            flags: Flags {
                parse_all_lines: switch(args.all_lines, config.all_lines),
                lossless: switch(args.lossless, config.lossless),
                output_tree: args
                    .output_tree
                    .or(config.output_tree)
                    .unwrap_or(OutputTree::Db),
                obfuscate_passwords: switch(args.obfuscate_passwords, config.obfuscate_passwords),
                script_extension: args
                    .script_extension
                    .or(config.script_extension)
                    .unwrap_or_else(|| "txt".to_string()),
            },
            config_path,
        })
    }
}

/// Accepts a file extension with or without a leading dot, e.g. `.fmscript` or `fmscript`
fn parse_file_extension(value: &str) -> Result<String, String> {
    let extension = value.trim_start_matches('.');
    if extension.is_empty() || extension.contains(['/', '\\']) {
        return Err(format!("'{value}' is not a valid file extension"));
    }
    Ok(extension.to_string())
}

/// Resolves a path from the config file: `~/` is the home directory, relative paths are
/// relative to the config file's folder. `./` segments are dropped for readable output.
fn resolve_path(base_dir: &Path, path: &Path) -> PathBuf {
    let path = match path.strip_prefix("~") {
        Ok(rest) => match std::env::var_os("HOME") {
            Some(home) => PathBuf::from(home).join(rest),
            None => path.to_path_buf(),
        },
        Err(_) => path.to_path_buf(),
    };
    base_dir
        .join(path)
        .components()
        .filter(|component| !matches!(component, Component::CurDir))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use clap::Parser;

    use super::*;

    fn args(args: &[&str]) -> Result<Args, clap::Error> {
        Args::try_parse_from([&["fm-xml-export-exploder"], args].concat())
    }

    fn parse_config(toml: &str) -> ConfigFile {
        ConfigFile::parse(toml, Path::new("/install")).unwrap()
    }

    #[test]
    fn output_tree_accepts_both_spellings() {
        for flag in ["--output-tree", "--output_tree", "-t"] {
            let args = args(&[flag, "domain", "source", "target"]).unwrap();
            assert!(
                matches!(args.output_tree, Some(OutputTree::Domain)),
                "{flag}"
            );
        }
    }

    #[test]
    fn script_extension_is_normalized_and_validated() {
        for value in ["fmscript", ".fmscript"] {
            let args = args(&["--script-extension", value, "s", "t"]).unwrap();
            assert_eq!(args.script_extension.as_deref(), Some("fmscript"));
        }
        for invalid in ["", ".", "a/b", "a\\b"] {
            assert!(
                args(&["--script-extension", invalid, "s", "t"]).is_err(),
                "{invalid:?}"
            );
        }
    }

    #[test]
    fn defaults_without_config() {
        let settings =
            Settings::combine(args(&["src", "out"]).unwrap(), ConfigFile::default(), None).unwrap();
        assert_eq!(settings.source, PathBuf::from("src"));
        assert_eq!(settings.target, PathBuf::from("out"));
        assert!(!settings.git);
        assert!(matches!(settings.flags.output_tree, OutputTree::Db));
        assert_eq!(settings.flags.script_extension, "txt");
        assert!(!settings.flags.obfuscate_passwords);
    }

    #[test]
    fn config_paths_are_relative_to_the_config_file() {
        let config = parse_config(
            r#"
            source = "./xml"
            target = "repos"
            "#,
        );
        assert_eq!(config.source, Some(PathBuf::from("/install/xml")));
        assert_eq!(config.target, Some(PathBuf::from("/install/repos")));
        assert_eq!(
            parse_config(r#"source = "/data/xml""#).source,
            Some(PathBuf::from("/data/xml"))
        );
    }

    #[test]
    fn command_line_wins_over_config() {
        let config = parse_config(
            r#"
            source = "./xml"
            target = "./repos"
            output_tree = "domain"
            git = true
            script_extension = ".fmscript"
            "#,
        );
        let settings = Settings::combine(
            args(&["--output-tree", "db", "--script-extension", "txt", "other"]).unwrap(),
            config,
            None,
        )
        .unwrap();
        assert_eq!(settings.source, PathBuf::from("other"));
        assert_eq!(settings.target, PathBuf::from("/install/repos"));
        assert!(matches!(settings.flags.output_tree, OutputTree::Db));
        assert_eq!(settings.flags.script_extension, "txt");
        assert!(settings.git);
    }

    #[test]
    fn invalid_config_is_rejected() {
        for invalid in [
            r#"outputtree = "domain""#,
            r#"output_tree = "both""#,
            r#"git = "yes""#,
            r#"script_extension = "a/b""#,
        ] {
            assert!(
                ConfigFile::parse(invalid, Path::new("/")).is_err(),
                "{invalid}"
            );
        }
    }

    #[test]
    fn missing_source_is_explained() {
        let error = Settings::combine(args(&[]).unwrap(), ConfigFile::default(), None)
            .err()
            .unwrap();
        assert!(error.to_string().contains("Missing source directory"));
    }

    /// The config file created by install.sh must stay valid
    #[test]
    fn install_script_config_template() {
        let install = fs::read_to_string("install.sh").unwrap();
        let start = install
            .find("<< 'CONFIG'\n")
            .expect("config template in install.sh");
        let template = &install[start + "<< 'CONFIG'\n".len()..];
        let template = &template[..template.find("\nCONFIG\n").unwrap()];

        let config = ConfigFile::parse(template, Path::new("/install")).unwrap();
        assert_eq!(config.source, Some(PathBuf::from("/install/xml")));
        assert_eq!(config.target, Some(PathBuf::from("/install/repos")));
        assert!(matches!(config.output_tree, Some(OutputTree::Domain)));
        assert_eq!(config.git, Some(true));
        assert_eq!(config.obfuscate_passwords, Some(true));
    }
}
