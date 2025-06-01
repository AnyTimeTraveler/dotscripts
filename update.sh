#!/usr/bin/env sh

cargo build --release --bins

RUST_LOG=info ./target/release/releaser

cp -v ./shell/*.sh $HOME/.local/bin/
cp -v ./python/*.py $HOME/.local/bin/
