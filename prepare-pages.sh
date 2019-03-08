#!/usr/bin/env sh

set -e -u

mkdir output
cargo run --release --bin=rustup-available-packages-html -- render -c config.yaml
ln -s x86_64-unknown-linux-gnu.html output/index.html
