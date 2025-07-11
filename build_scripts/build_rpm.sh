#!/usr/bin/bash

cd "$(dirname "$(realpath "$0")")/.." || exit

bash ./build_scripts/build_linux.sh
cargo install cargo-generate-rpm
cargo generate-rpm