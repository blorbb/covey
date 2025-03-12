#!/usr/bin/env bash

SCRIPT_PATH=$(dirname "$(realpath -s "$0")")
cd "$SCRIPT_PATH"
cd ..

cargo publish -p "covey-proto"
cargo publish -p "covey-schema"
cargo publish -p "covey-manifest-macros"
cargo publish -p "covey-plugin"
cargo publish -p "covey"
