#! /bin/bash

# Script to generate JS types

# Clean up the generated files
# Needed otherwise we don't detect deleted types
pnpm run clean-generated-files

# Execute only tests for "export bindings" (it's the prefix used by ts-rs)
# (ts-rs uses Rust tests to generate the types)
cargo test export_bindings_

# Format the files with Biome
pnpm run generate-index-files
