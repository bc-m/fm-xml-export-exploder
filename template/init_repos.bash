#!/bin/bash

folders=(
    "custom_functions"
    "external_data_sources"
    "layouts"
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
