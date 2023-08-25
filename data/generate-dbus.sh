#!/bin/bash

set -euo pipefail

data_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
src_dir="$data_dir/../src"

cargo install dbus-codegen

dbus-codegen-rust --system-bus \
	-d net.hadess.PowerProfiles \
	-p /net/hadess/PowerProfiles \
	-o "$src_dir/powerprofiles.rs"
