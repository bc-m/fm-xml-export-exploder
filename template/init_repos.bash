#!/bin/bash

folders=(
    "tables"
    "scripts"
    "scripts_sanitized"
    "layouts"
    "custom_functions"
    "external_data_sources"
    "value_lists"
    "themes"
)

for folder in "${folders[@]}"; do
    mkdir -p "./repos/$folder"
    git -C "./repos/$folder" init
    touch "./repos/$folder/.gitignore"
    echo ".DS_Store" > "./repos/$folder/.gitignore"
done

echo "Done!"
