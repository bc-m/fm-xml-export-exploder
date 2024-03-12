#!/bin/bash
set -e

echo "Start!"
start=$(date +'%Y-%m-%d %H:%M:%S')

# Open a FileMaker file that performs an XML export on startup and exists after it
echo "Start FileMaker XML-export..."
open -a "FileMaker Pro" "./Run_XML_Export.fmp12"

# Wait for FileMaker to exit
echo -n "Waiting for FileMaker XML-export to finish..."
while pgrep -u "$(whoami)" -x "FileMaker Pro" > /dev/null; do
    echo -n "."
    sleep 5
done
echo
echo "FileMaker has exited"

# Explode XML using fm-xml-export-exploder
echo "Exploding XML..."
./bin/fm-xml-export-exploder ./xml ./repo

set +e

# Commit and push changes for each folder
# Define an array of folders
folders=(
    "custom_functions"
    "custom_menu_sets"
    "custom_menus"
    "extended_privileges"
    "external_data_sources"
    "layouts"
    "privilege_sets"
    "relationships"
    "scripts"
    "scripts_sanitized"
    "table_occurrences"
    "tables"
    "themes"
    "value_lists"
)

# Function to commit and push changes
commit_message = $start
commit_and_push() {
    local folder="$1"
    echo "Commit $folder..."
    git -C "./repo/$folder" add --all
    git -C "./repo/$folder" commit -m "$commit_message"
    git -C "./repo/$folder" push
}

for folder in "${folders[@]}"; do
    commit_and_push "$folder"
done

echo "Done!"
