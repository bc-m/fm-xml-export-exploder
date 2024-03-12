#!/bin/bash

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

for folder in "${folders[@]}"; do
    mkdir -p "./repos/$folder"
    git -C "./repos/$folder" init
    touch "./repos/$folder/.gitignore"
    echo ".DS_Store" > "./repos/$folder/.gitignore"
done

echo "Done!"
