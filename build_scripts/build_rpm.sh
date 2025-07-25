#!/usr/bin/bash

# Change dir to the cargo project
cd "$(dirname "$(realpath "$0")")/.." || exit

RUST_TARGET=${1:-"x86_64-unknown-linux-gnu"}

bash ./build_scripts/build_linux.sh "$RUST_TARGET"
cargo install cargo-generate-rpm
cargo generate-rpm

rpm --addsign ./target/generate-rpm/*.rpm