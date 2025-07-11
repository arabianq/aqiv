#!/usr/bin/sh

sh ./build_linux.sh
cargo install cargo-deb
cargo deb