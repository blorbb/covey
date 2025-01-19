use std::path::Path;

fn main() {
    comette_tauri_types::export_ts_to(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("src")
            .join("lib")
            .join("bindings"),
    );
    tauri_build::build()
}
