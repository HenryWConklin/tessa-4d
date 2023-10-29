#!/bin/bash
set -e

CARGO_PROJECTS=('tessa4d' 'tessa4d-bevy' 'tessa4d-gdext')

# Run tests first so formatting/lints don't block it.
for project in "${CARGO_PROJECTS[@]}"; do
  cargo test --package "$project"
done

rustup component add clippy
rustup component add rustfmt
for project in "${CARGO_PROJECTS[@]}"; do
  cargo clippy --package "$project" --no-deps
  cargo fmt --check --package "$project"
done
