#!/usr/bin/bash

cd "$(dirname "$(realpath "$0")")/.." || exit

bash ./build_scripts/build_deb.sh
bash ./build_scripts/build_rpm.sh
bash ./build_scripts/build_windows.sh