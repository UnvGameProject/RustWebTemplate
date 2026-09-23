#!/bin/sh
set -eu

cd /workspace
mkdir -p generated

# The first run creates a lockfile from exact top-level versions. Every later
# run is reproducible through npm ci. Lifecycle scripts are disabled because
# neither Bootstrap nor Sass requires them for this POC.
if [ ! -f package-lock.json ]; then
    npm install --package-lock-only --ignore-scripts --no-audit --no-fund
fi

npm ci --ignore-scripts --no-audit --no-fund
npm run build:assets
exec npm run watch:css
