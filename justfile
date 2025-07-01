# run and watch covey-tauri
dev:
    cd ./covey-tauri && pnpm tauri dev

# build covey-tauri app in release mode
build:
    cd ./covey-tauri && NOSTRIP=true pnpm tauri build

install-plugins mode="release":
    just --justfile="../covey-plugins/justfile" install-all {{mode}}

# publish all packages onto crates.io
publish:
    cargo publish -p "covey-proto"
    cargo publish -p "covey-schema"
    cargo publish -p "covey-manifest-macros"
    cargo publish -p "covey-plugin"
    cargo publish -p "covey"
