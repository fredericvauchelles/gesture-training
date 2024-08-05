#!/bin/bash

cd /data || exit

echo "Building Binary"
cargo build --release --package gesture_training

echo "Building cargo-sources.json"
python3 /flatpak-cargo-generator.py Cargo.lock -o target/cargo-sources.json

