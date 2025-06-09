#!/usr/bin/env sh

set -e

cd "$(dirname "$0")"

cargo build --release --bins

./target/release/releaser
