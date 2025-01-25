use std::fmt;

#[doc(hidden)]
/// Private implementation details. Do not use.
pub mod __private_generation {
    pub use covey_manifest_macro::generate_config;
    pub use serde;
}

#[macro_export]
macro_rules! generate_config {
    ($path:literal) => {
        $crate::manifest::__private_generation::generate_config!(
            file = $path,
            serde_path = $crate::manifest::__private_generation::serde
        );

        impl $crate::manifest::ManifestDeserialization for self::Config {
            fn try_from_input(s: &str) -> Result<Self, $crate::manifest::DeserializationError> {
                toml::from_str(s)
                    .map_err(|e| $crate::manifest::DeserializationError(e.message().to_string()))
            }
        }
    };
    () => {
        $crate::generate_config!("./manifest.toml");
    };
}

/// Automatically implemented by [`generate_config!`].
///
/// The unit type `()` also implements this, ignoring the input.
pub trait ManifestDeserialization: Sized {
    /// Constructs `Self` from the user's plugin configuration.
    ///
    /// The input string is currently in TOML format. This may change
    /// in the future.
    fn try_from_input(s: &str) -> Result<Self, DeserializationError>;
}

impl ManifestDeserialization for () {
    /// Ignores the input string and always succeeds.
    fn try_from_input(_: &str) -> Result<Self, DeserializationError> {
        Ok(())
    }
}

/// Error obtained from deserializing the user's plugin configuration.
#[derive(Debug, Clone)]
pub struct DeserializationError(pub String);

impl fmt::Display for DeserializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("failed to deserialize plugin configuration: ")?;
        f.write_str(&self.0)?;
        Ok(())
    }
}

impl std::error::Error for DeserializationError {}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn expanded_types_exist() {
        mod config {
            use crate::manifest::__private_generation;

            __private_generation::generate_config!(
                serde_path = crate::manifest::__private_generation::serde,
                inline = r#"
                    name = "Open"
                    description = "Open URLs with a query"
                    repository = "https://github.com/blorbb/covey-plugins"
                    authors = ["blorbb"]

                    [schema.urls]
                    title = "List of URLs to show"
                    type.map.value-type.struct.fields = { name = "str", url = "str" }
                "#
            );
        }

        config::Config {
            urls: HashMap::from([(
                "key".to_string(),
                config::urls::UrlsValue {
                    name: "name".to_string(),
                    url: "urls".to_string(),
                },
            )]),
        };
    }
}
