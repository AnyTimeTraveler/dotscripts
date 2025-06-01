#!/usr/bin/env sh

set -e

cd "$(dirname "$0")"

cargo build --release --bins

RUST_LOG=info ./target/release/releaser

cp -v ./shell/*.sh $HOME/.local/bin/
cp -v ./python/*.py $HOME/.local/bin/
