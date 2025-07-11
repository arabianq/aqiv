#!/usr/bin/bash

bash ./build_linux.sh
cargo install cargo-generate-rpm
cargo generate-rpm