#!/usr/bin/bash

cd "$(dirname "$(realpath "$0")")/.." || exit

bash ./build_scripts/build_deb.sh
bash ./build_scripts/build_rpm.sh
bash ./build_scripts/build_windows.sh

mkdir ./target/for_github_release
cp "$(find ./target/x86_64-unknown-linux-gnu/debian/aqiv_*_amd64.deb | sort -V | tail -n 1)" ./target/for_github_release/
cp "$(find ./target/generate-rpm/aqiv-*.x86_64.rpm | sort -V | tail -n 1)" ./target/for_github_release/
cp ./target/x86_64-unknown-linux-gnu/release/aqiv ./target/for_github_release/aqiv-x86_64-linux-gnu
cp ./target/x86_64-pc-windows-gnu/release/aqiv.exe ./target/for_github_release/aqiv-x86_64-windows-gnu.exe