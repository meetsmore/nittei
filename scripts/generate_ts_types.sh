#! /bin/bash

# Script to generate JS types

# Clean up the generated files
pnpm run clean-generated-files

# Execute only "export bindings" test (it's the prefix used by ts-rs)
cargo test export_bindings_

# Format the files
pnpm run generate-index-files
