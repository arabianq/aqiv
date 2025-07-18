#!/usr/bin/bash

# Clean all cargo files
cargo clean

# Change dir to the cargo project
cd "$(dirname "$(realpath "$0")")/.." || exit

RUST_TARGET_LINUX="x86_64-unknown-linux-gnu"
RUST_TARGET_WINDOWS="x86_64-pc-windows-gnu"

# Build all packages
bash ./build_scripts/build_deb.sh "$RUST_TARGET_LINUX"
bash ./build_scripts/build_rpm.sh "$RUST_TARGET_LINUX"
bash ./build_scripts/build_windows.sh "$RUST_TARGET_WINDOWS"

# Move .deb and .rpm packages to for_github_release directory
mkdir ./target/for_github_release
cp "$(find ./target/$RUST_TARGET_LINUX/debian/aqiv_*_amd64.deb | sort -V | tail -n 1)" ./target/for_github_release/
cp "$(find ./target/generate-rpm/aqiv-*.x86_64.rpm | sort -V | tail -n 1)" ./target/for_github_release/

# Compress Windows and Linux binaries using upx (if upx is installed)
if command -v upx >/dev/null 2>&1; then
  upx --best ./target/$RUST_TARGET_LINUX/release/aqiv
  upx -t ./target/$RUST_TARGET_LINUX/release/aqiv

  upx --best ./target/$RUST_TARGET_WINDOWS/release/aqiv.exe
  upx -t ./target/$RUST_TARGET_WINDOWS/release/aqiv.exe
fi

# Move Linux and Windows binaries to for_github_release directory
cp ./target/$RUST_TARGET_LINUX/release/aqiv ./target/for_github_release/aqiv-x86_64-linux-gnu
cp ./target/$RUST_TARGET_WINDOWS/release/aqiv.exe ./target/for_github_release/aqiv-x86_64-windows-gnu.exe