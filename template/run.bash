#!/bin/bash
set -e

# Open a FileMaker file that performs an XML export on startup and quits afterwards
echo "Start FileMaker XML export..."
open -a "FileMaker Pro" "./Run_XML_Export.fmp12"
while pgrep -u "$(whoami)" -x "FileMaker Pro" > /dev/null; do
    sleep 5
done

# Explode and commit: one git repository per domain folder (scripts, tables, ...)
./bin/fm-xml-export-exploder --git -t domain ./xml ./repos

# Push every repository that has a remote
for repo in ./repos/*/; do
    if [ -n "$(git -C "$repo" remote)" ]; then
        git -C "$repo" push
    fi
done
