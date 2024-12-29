#[doc(hidden)]
/// Private implementation details. Do not use.
pub mod __private_generation {
    pub use qpmu_manifest_macro::generate_config;
    pub use serde;
}

#[macro_export]
macro_rules! generate_config {
    ($path:literal) => {
        $crate::manifest::__private_generation::generate_config!(
            file = $path,
            serde_path = $crate::manifest::__private_generation::serde
        );
    };
    () => {
        $crate::generate_config!("./manifest.toml");
    }
}
