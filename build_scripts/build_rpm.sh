#!/usr/bin/sh

sh ./build_linux.sh
cargo install cargo-generate-rpm
cargo generate-rpm