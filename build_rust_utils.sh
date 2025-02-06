#!/usr/bin/env sh

cargo +nightly -Z unstable-options build --artifact-dir . --release --bins --all-features
