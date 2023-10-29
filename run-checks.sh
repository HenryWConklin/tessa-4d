#!/bin/bash
set -e

CARGO_PROJECTS=('tessa4d' 'tessa4d-bevy' 'tessa4d-gdext')

# Run tests first so formatting/lints don't block it.
for project in "${CARGO_PROJECTS[@]}"; do
  echo "Test ${project}"
  cargo test --package "$project"
done

for project in "${CARGO_PROJECTS[@]}"; do
  echo "Clippy ${project}"
  cargo clippy --package "$project" --no-deps
  echo "Rustfmt ${project}"
  cargo fmt --check --package "$project"
done
