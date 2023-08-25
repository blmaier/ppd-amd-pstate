#!/bin/bash

set -euo pipefail

data_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
src_dir="$data_dir/../src"
xml=$data_dir/net.hadess.PowerProfiles.xml

curl -o "$xml" "https://gitlab.freedesktop.org/hadess/power-profiles-daemon/-/raw/main/src/net.hadess.PowerProfiles.xml"

cargo install dbus-codegen

dbus-codegen-rust --system-bus --file "$xml" -o "$src_dir/powerprofiles.rs"
