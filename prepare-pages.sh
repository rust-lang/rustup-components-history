#!/usr/bin/env sh

# Fail on errors
set -e -u

# Build the project
cargo build --verbose --all

# Test everything
cargo test --verbose --all

# Generate the pages
mkdir output
cargo run --release --bin=rustup-available-packages-html -- render -c config.yaml
ln -s x86_64-unknown-linux-gnu.html output/index.html
