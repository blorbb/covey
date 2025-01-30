use std::fmt;

#[doc(hidden)]
/// Private implementation details. Do not use.
pub mod __private_generation {
    pub use covey_manifest_macro::include_manifest;
    pub use serde;
    pub use serde_json;
}

#[macro_export]
macro_rules! include_manifest {
    ($path:literal) => {
        $crate::manifest::__private_generation::include_manifest!(
            file = $path,
            // this can't be $crate as it's used by serde, requires a proper path
            serde_path = covey_plugin::manifest::__private_generation::serde,
            ext_impl_ty = $crate::ListItem,
            command_return_ty = $crate::Result<R>,
            command_return_trait = ::core::convert::Into<$crate::Actions>,
        );

        impl $crate::manifest::ManifestDeserialization for self::Config {
            fn try_from_input(s: &str) -> Result<Self, $crate::manifest::DeserializationError> {
                $crate::manifest::__private_generation::serde_json::from_str(s)
                    .map_err(|e| $crate::manifest::DeserializationError(e.to_string()))
            }
        }
    };
    () => {
        $crate::include_manifest!("./manifest.toml");
    };
}

/// Automatically implemented by [`include_manifest!`].
///
/// The unit type `()` also implements this, ignoring the input.
pub trait ManifestDeserialization: Sized {
    /// Constructs `Self` from the user's plugin configuration.
    ///
    /// The input string is currently in JSON format. This may change
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

            __private_generation::include_manifest!(
                serde_path = crate::manifest::__private_generation::serde,
                ext_impl_ty = crate::ListItem,
                command_return_ty = crate::Result<R>,
                command_return_trait = ::core::convert::Into<crate::Actions>,
                inline = r#"
                    name = "Open"
                    description = "Open URLs with a query"
                    repository = "https://github.com/blorbb/covey-plugins"
                    authors = ["blorbb"]

                    [[schema]]
                    id = "urls"
                    title = "List of URLs to show"
                    type.map.value-type.struct.fields = { name = "text", url = "text" }
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

        use config::CommandExt;
        crate::ListItem::new("ajwroiajw").on_activate(|| async { Ok(vec![]) });
    }
}
