# FileMaker XML Export Exploder

Turns FileMaker solutions saved as XML into a folder of small, readable files: one file per script, layout, table, custom function, value list and so on, organized like the folders in FileMaker. Scripts are additionally written as plain text, the way they appear in the FileMaker Script Workspace. Together with git, you get a reviewable history of every change to your solution.

Example: from [these XML exports](./tests/xml) you get [this output](./tests/snapshots_db_lossless), with scripts as readable as [this one](./tests/snapshots_db_lossless/fmSyntaxColorizer/scripts_sanitized/Example%20SyntaxColorizing%20-%20ID%2046/All%20script%20steps%20and%20all%20options%20-%20ID%2011.txt.snap).

## Features

- **Readable scripts and custom functions** in `scripts_sanitized` and `custom_functions_sanitized`, formatted like the FileMaker Script Workspace, including all script step parameters.
- **One file per item**, in folders mirroring the folders of your scripts, layouts and custom functions.
- **Built for git:** noise like UUIDs and timestamps is removed, unchanged files are not rewritten, and `--git` commits the output for you.
- **Fast:** written in Rust and processing files in parallel. On a current Mac, a 950 MB export of 76 files takes about 5 seconds, re-runs about 3 seconds.
- **Two layouts:** organize the output by database (default) or by domain (`scripts/`, `tables/`, …).

## Installation

**Automated exports on macOS:** the [install script](#automated-exports-on-macos) downloads the latest version and sets everything up.

**Manual:** download the binary for your system from the [releases page](https://github.com/bc-m/fm-xml-export-exploder/releases/latest) (macOS, Linux and Windows, Intel and ARM). On macOS, the first start of a downloaded binary is blocked by Gatekeeper: right-click it in Finder and choose "Open" once.

## Usage

1. Save your FileMaker solution as XML with [Save a Copy as XML](https://help.claris.com/en/pro-help/content/save-a-copy-as-xml.html) into a folder.
2. Run:

```bash
fm-xml-export-exploder [OPTIONS] <SOURCE> <TARGET>
```

`SOURCE` is the folder with the XML files, `TARGET` the folder for the output. Each run updates the output in place: changed files are rewritten, unchanged files are left alone, and files of items that no longer exist are removed.

Exports split into one file per catalog (FileMaker 2026) work as well: put the summary file (e.g. `Datenbankbericht.xml`) and the folder it refers to in `SOURCE`. The catalog files in that folder are exploded together as one database. Since every catalog file has its own DDR_INFO, the files in `_` are named after the catalog, e.g. `ddr_info_ScriptCatalog.xml` (and `skeleton_ScriptCatalog.xml` with `--lossless`).

| Option | Description |
|---|---|
| `-t`, `--output-tree <db\|domain>` | Organize the output by database (`db`, default) or by domain (`domain`), see [Output](#output). `--output_tree` still works. |
| `-c`, `--config <file>` | Read options from a config file, see [Config file](#config-file) |
| `--git` | Commit the output with git, see below |
| `--obfuscate-passwords` | Replace passwords in readable scripts with `••••••••` |
| `--script-extension <ext>` | File extension of the readable scripts in `scripts_sanitized` (default `txt`), e.g. `fmscript` for syntax highlighting in your editor |
| `-a`, `--all-lines` | Keep lines that are skipped by default to reduce noise |
| `-l`, `--lossless` | Keep all information of the XML, including less important catalogs and attributes, so the export can be [reassembled](#reassembling-an-export) |

### Config file

Instead of passing options every time, put them into a `fm-xml-export-exploder.toml`. It is used automatically when it is in the current directory, or explicitly with `--config <file>`. Then `fm-xml-export-exploder` runs without arguments:

```toml
source = "./xml"         # folder with the XML export
target = "./repos"       # folder for the output
output_tree = "domain"   # "db" or "domain"
git = true
script_extension = "txt"
obfuscate_passwords = true
all_lines = false
lossless = false
```

All entries are optional. Options on the command line override the config file, and relative paths are relative to the folder of the config file. The [install script](#automated-exports-on-macos) creates a config file with a comment for every option.

### Versioning the output with git

With `--git`, the output is committed after each run:

- With `-t db` (default) each database folder becomes its own repository, with `-t domain` each domain folder (e.g. `scripts/`, `tables/`).
- A commit (`Export <database>`) is only created when the output changed.
- The installed `git` is used, so your git configuration (author, signing, default branch) applies. Without git, the output is written but not committed.
- Folders that are already inside another git repository (e.g. an output folder you version yourself) are left alone.

To share the history, add a remote to a repository once (`git -C <repository> remote add origin <url>`) and push as usual.

### Reassembling an export

Output exploded with `--lossless` can be put back together into the XML files it came from:

```bash
fm-xml-export-exploder reassemble <SOURCE> <TARGET>
```

`SOURCE` is the output folder of the explode run, organized by database or by domain, `TARGET` the folder for the XML files. Each database becomes `<database>.xml`, and an export split into catalogs becomes one `<database>_<catalog>.xml` per catalog file (the summary file is not recreated). The reassembled files are identical to the exported ones, written as UTF-8 without BOM: FileMaker 22 and earlier export UTF-16, so for those files the content is identical, not the bytes.

Output exploded without `--lossless` has no skeletons and can't be reassembled. If you keep the output in git, make sure git doesn't convert line endings (e.g. `* -text` in `.gitattributes`), or the reassembled files differ from the export.

## Automated exports on macOS

The install script sets up a folder that exports your solution, explodes it and commits the result, optionally every day:

```bash
curl -fsSL https://raw.githubusercontent.com/bc-m/fm-xml-export-exploder/main/install.sh | bash
```

It asks where to install, suggesting a new folder `fm-xml-export-exploder` in the current directory. To skip the question or to also schedule a daily export at 7:00, pass options:

```bash
curl -fsSL https://raw.githubusercontent.com/bc-m/fm-xml-export-exploder/main/install.sh | bash -s -- --service ~/fm-export
```

Then create `Run_XML_Export.fmp12` in that folder: a FileMaker file whose `OnFirstWindowOpen` script saves your solution as XML into the `xml` folder and quits FileMaker. Tip: [Save a Copy as XML](https://help.claris.com/en/pro-help/content/save-a-copy-as-xml.html) works on any file that is open with full access, addressed by its window name.

The folder then contains:

```
├── bin/                    the exploder
├── xml/                    the XML export
├── repos/                  the output, one git repository per domain
├── log/                    output of the daily export
├── fm-xml-export-exploder.toml   the options, with a comment for each
├── run.bash                export, explode, commit, and push repositories that have a remote
└── com.fm-xml-export-exploder.plist   the daily export (installed with --service)
```

Change the options in `fm-xml-export-exploder.toml` and run `run.bash` to export manually. To update, run the install script again with the same folder: it replaces the binary, `run.bash` and the service file (a changed `run.bash` or service file is kept as `.bak`) and leaves the config, `xml/`, `repos/`, `log/` and `Run_XML_Export.fmp12` alone. A daily export set up earlier is updated as well.

## Output

### By database (default, `-t db`)

```
└── <database>/
    ├── scripts/
    │   └── <folder> - ID <folder id>/
    │       └── <script> - ID <script id>.xml
    ├── scripts_sanitized/
    │   └── <folder> - ID <folder id>/
    │       └── <script> - ID <script id>.txt
    ├── tables/
    │   └── <table> - ID <table id>.xml
    ├── table_occurrences/
    │   └── <table occurrence> - ID <table occurrence id>.xml
    ├── themes/
    │   └── <theme> - ID <theme id>.xml
    ├── value_lists/
    │   └── <value list> - ID <value list id>.xml
    └── …
```

### By domain (`-t domain`)

```
├── scripts/
│   └── <database>/
│       └── <folder> - ID <folder id>/
│           └── <script> - ID <script id>.xml
├── scripts_sanitized/
│   └── <database>/
│       └── <folder> - ID <folder id>/
│           └── <script> - ID <script id>.txt
├── tables/
│   └── <database>/
│       └── <table> - ID <table id>.xml
└── …
```

### Why this structure?

Git does not work well with large files or a large *number* of files. The domain layout lets you keep a separate repository per domain, e.g. only `scripts_sanitized` to review script changes in a readable format. For small solutions, one repository per database works just as well.

## Development

```bash
cargo test
```

The snapshot tests explode the solutions in [`tests/xml`](./tests/xml) and compare the result with [`tests/snapshots_*`](./tests). After an intended output change, update the snapshots with `INSTA_UPDATE=always cargo test` (or `cargo insta review`) and review the diff.

Publishing a GitHub release builds and attaches the binaries for all platforms.

### Test data

These open source FileMaker solutions are used for the snapshot tests:

- [FM-Admin-API-Tool](https://github.com/SoliantMike/FM-Admin-API-Tool): a FileMaker tool for administering FileMaker Server using the Admin API.
- [fmSyntaxColorizer](https://github.com/mrwatson-de/fmSyntaxColorizer): a FileMaker tool for customizing syntax highlighting for FileMaker scripts and calculations.
- [ooe-fm](https://github.com/mislavkos/ooe-fm/): a sample FileMaker file with One Of Everything (Ooe).

## Contributing

Contributions are welcome! Feel free to open issues or pull requests on [GitHub](https://github.com/bc-m/fm-xml-export-exploder).

## License

MIT, see [LICENSE](LICENSE).
