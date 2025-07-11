#!/usr/bin/bash

bash ./build_linux.sh
cargo install cargo-deb
cargo deb