#!/usr/bin/bash

cd "$(dirname "$(realpath "$0")")/.." || exit

bash ./build_scripts/build_linux.sh
cargo install cargo-deb
cargo deb --target x86_64-unknown-linux-gnu