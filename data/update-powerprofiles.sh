#!/bin/bash

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)

curl -o "$script_dir/net.hadess.PowerProfiles.xml" "https://gitlab.freedesktop.org/hadess/power-profiles-daemon/-/raw/main/src/net.hadess.PowerProfiles.xml"
