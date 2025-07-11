#!/usr/bin/bash

cd "$(dirname "$(realpath "$0")")/.." || exit

rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu