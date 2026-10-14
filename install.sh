#!/bin/bash
#
# Installs or updates fm-xml-export-exploder with a setup for automated FileMaker XML
# exports on macOS:
#
#   curl -fsSL https://raw.githubusercontent.com/bc-m/fm-xml-export-exploder/main/install.sh | bash
#
# It asks where to install (default: ./fm-xml-export-exploder). Options can be passed
# with `| bash -s -- [OPTIONS] [DIRECTORY]`:
#   DIRECTORY   Where to install, without asking
#   --service   Also run the export every day at 7:00 (launchd)
#
# Installing into an existing folder updates it: the binary, run.bash and the service
# file are replaced (changed files are kept as .bak), the data in xml/, repos/ and log/
# and Run_XML_Export.fmp12 are left alone.

set -euo pipefail

REPO="bc-m/fm-xml-export-exploder"
BINARY="fm-xml-export-exploder"
SERVICE_LABEL="com.fm-xml-export-exploder"

dir=""
install_service=false
for arg in "$@"; do
    case "$arg" in
        --service) install_service=true ;;
        -h | --help)
            echo "Usage: install.sh [--service] [DIRECTORY]"
            echo "  DIRECTORY   Where to install (default: ask, suggesting ./$BINARY)"
            echo "  --service   Also run the export every day at 7:00 (launchd)"
            exit 0
            ;;
        -*)
            echo "Unknown option: $arg" >&2
            exit 1
            ;;
        *) dir="$arg" ;;
    esac
done

if [ "$(uname -s)" != "Darwin" ]; then
    echo "This setup uses FileMaker Pro and launchd and is for macOS only." >&2
    echo "On other systems, download the binary from https://github.com/$REPO/releases/latest" >&2
    exit 1
fi

if [ -z "$dir" ]; then
    dir="$PWD/$BINARY"
    # stdin is the script itself with `curl | bash`, so ask on the terminal. Without
    # one (e.g. in automation) the default is used.
    if { exec 3< /dev/tty; } 2> /dev/null; then
        read -r -u 3 -p "Install to [$dir]: " answer || true
        exec 3<&-
        if [ -n "${answer:-}" ]; then
            dir="${answer/#\~/$HOME}"
        fi
    fi
fi

if [ -d "$dir" ]; then
    echo "Updating the existing installation in $dir"
fi
mkdir -p "$dir"/{bin,xml,repos,log}
dir="$(cd "$dir" && pwd)"

# Writes stdin to a file; an existing file with other content is kept as .bak
install_file() {
    local target="$1" new
    new="$(mktemp)"
    cat > "$new"
    if [ ! -e "$target" ]; then
        echo "Created $target"
    elif cmp -s "$new" "$target"; then
        rm "$new"
        return
    else
        cp "$target" "$target.bak"
        echo "Updated $target (previous version kept as $target.bak)"
    fi
    mv "$new" "$target"
}

echo "Downloading the latest $BINARY..."
curl -fsSL "https://github.com/$REPO/releases/latest/download/$BINARY-universal-apple-darwin.tar.gz" |
    tar -xz -C "$dir/bin"
echo "Installed $("$dir/bin/$BINARY" --version) to $dir/bin"

install_file "$dir/run.bash" << 'EOF'
#!/bin/bash
#
# Exports the FileMaker solution as XML, explodes it and commits the result with git.
# Run_XML_Export.fmp12 has to save the solution as XML into ./xml and quit FileMaker.

set -e
cd "$(dirname "$0")"

if [ ! -e Run_XML_Export.fmp12 ]; then
    echo "Missing Run_XML_Export.fmp12, see https://github.com/bc-m/fm-xml-export-exploder#automated-exports-on-macos" >&2
    exit 1
fi

echo "$(date '+%Y-%m-%d %H:%M:%S') Exporting the FileMaker solution as XML..."
# -W waits until FileMaker quits
open -W -a "FileMaker Pro" Run_XML_Export.fmp12

# One git repository per domain folder (scripts, tables, ...)
./bin/fm-xml-export-exploder --git -t domain ./xml ./repos

# Push every repository that has a remote
for repo in ./repos/*/; do
    if [ -n "$(git -C "$repo" remote)" ]; then
        git -C "$repo" push
    fi
done

echo "$(date '+%Y-%m-%d %H:%M:%S') Done."
EOF
chmod +x "$dir/run.bash"

service_file="$dir/$SERVICE_LABEL.plist"
install_file "$service_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$SERVICE_LABEL</string>
    <key>ProgramArguments</key>
    <array>
        <string>/bin/bash</string>
        <string>$dir/run.bash</string>
    </array>
    <key>WorkingDirectory</key>
    <string>$dir</string>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Hour</key>
        <integer>7</integer>
        <key>Minute</key>
        <integer>0</integer>
    </dict>
    <key>StandardOutPath</key>
    <string>$dir/log/out.log</string>
    <key>StandardErrorPath</key>
    <string>$dir/log/error.log</string>
</dict>
</plist>
EOF

# A daily export set up earlier for this folder is updated as well
agent="$HOME/Library/LaunchAgents/$SERVICE_LABEL.plist"
if [ -e "$agent" ] && grep -qF "<string>$dir/run.bash</string>" "$agent"; then
    install_service=true
fi

if [ "$install_service" = true ]; then
    mkdir -p "$(dirname "$agent")"
    cp "$service_file" "$agent"
    # Replace a previously loaded version
    launchctl bootout "gui/$(id -u)" "$agent" 2> /dev/null || true
    launchctl bootstrap "gui/$(id -u)" "$agent"
    echo "Installed the daily export (7:00) as $agent"
fi

echo
echo "Next steps:"
if [ ! -e "$dir/Run_XML_Export.fmp12" ]; then
    echo "  - Create $dir/Run_XML_Export.fmp12: a FileMaker file whose OnFirstWindowOpen"
    echo "    script saves your solution as XML into $dir/xml and quits FileMaker."
fi
echo "  - Run $dir/run.bash to export, explode and commit."
if [ "$install_service" = false ]; then
    echo "  - Optional: run the installer again with --service for a daily export."
fi
