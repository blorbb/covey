use std::{
    collections::HashMap,
    fmt::{self, Debug},
    marker::PhantomData,
};

use indexmap::IndexMap;
use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, Visitor},
};

pub mod generate;
#[cfg(feature = "ts-rs")]
pub use ts_rs;

/// A manifest for a single plugin.
///
/// This should be a TOML file stored in
/// `data directory/comette/plugins/<plugin>/manifest.toml`.
#[derive(Debug, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(rename_all = "camelCase"))]
#[non_exhaustive]
#[serde(rename_all = "kebab-case")]
pub struct PluginManifest {
    /// User-facing name of the plugin.
    pub name: String,
    /// Plugin description. Can be multiple lines. Supports markdown.
    pub description: Option<String>,
    /// URL to the plugin's repository.
    pub repository: Option<String>,
    /// List of authors of the plugin.
    #[serde(default)]
    pub authors: Vec<String>,
    /// Key is the ID of the configuration option.
    #[serde(default)]
    pub schema: IndexMap<String, ConfigSchema>,
}

impl PluginManifest {
    pub fn try_from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(rename_all = "camelCase"))]
#[non_exhaustive]
pub struct ConfigSchema {
    pub title: String,
    pub description: Option<String>,
    pub r#type: ConfigType,
}

/// TODO: better docs
///
/// If there is no default, then this type will be *required*.
#[derive(Debug, PartialEq)]
#[cfg_attr(
    feature = "ts-rs",
    derive(ts_rs::TS),
    ts(rename_all = "camelCase", tag = "kind")
)]
pub enum ConfigType {
    Int(ConfigInt),
    Str(ConfigStr),
    Bool(ConfigBool),
    FilePath(ConfigFilePath),
    FolderPath(ConfigFolderPath),
    List(ConfigList),
    Map(ConfigMap),
    Struct(ConfigStruct),
}

// the below structs can't use the macro because they have extra
// required fields.
// all structs should have the same serde meta tag.

#[derive(Debug, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(rename_all = "camelCase"))]
#[serde(rename_all = "kebab-case")]
pub struct ConfigList {
    pub item_type: Box<ConfigType>,
    #[serde(default)]
    pub min_items: u64,
    /// Whether all items in the list must be unique.
    #[serde(default)]
    pub unique: bool,
}

#[derive(Debug, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(rename_all = "camelCase"))]
#[serde(rename_all = "kebab-case")]
/// A map from any string to a specified value.
pub struct ConfigMap {
    pub value_type: Box<ConfigType>,
    #[serde(default)]
    pub min_items: u64,
}

/// A map with specific key-value pairs.
#[derive(Debug, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(rename_all = "camelCase"))]
#[serde(rename_all = "kebab-case")]
pub struct ConfigStruct {
    pub fields: HashMap<String, ConfigType>,
}

/// A selection of one of multiple strings.
#[derive(Debug, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(rename_all = "camelCase"))]
#[serde(rename_all = "kebab-case")]
pub struct ConfigSelection {
    pub allowed_values: Vec<String>,
    #[serde(default)]
    pub default: Option<String>,
}

macros::make_config_subtypes! {
    pub struct ConfigInt {
        pub min: i64 = i64::MIN,
        pub max: i64 = i64::MAX,
        pub default: Option<i64> = None,
    }
    pub struct ConfigStr {
        pub min_length: u64 = u64::MIN,
        pub max_length: u64 = u64::MAX,
        pub default: Option<String> = None,
    }
    pub struct ConfigBool {
        pub default: Option<bool> = None,
    }
    pub struct ConfigFilePath {
        pub extension: Option<Vec<String>> = None,
        pub default: Option<String> = None,
    }
    pub struct ConfigFolderPath {
        pub default: Option<String> = None,
    }
}

/// Equivalent to [`ConfigType`] but with a derived deserialisation
/// implementation.
///
/// This is needed to avoid adding `#[deserialize_with = "string_or_struct"]`
/// on every instance of [`ConfigType`], and to be used in nested types like
/// a [`HashMap<_, ConfigType>`].
///
/// [`ConfigType`] has a manual deserialisation implementation that uses
/// the deserialisation of this.
///
/// [`ConfigType`] isn't a struct wrapper around this so that users can match
/// on it's variants.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum __ConfigTypeSerdeDerive {
    Int(ConfigInt),
    Str(ConfigStr),
    Bool(ConfigBool),
    FilePath(ConfigFilePath),
    FolderPath(ConfigFolderPath),
    List(ConfigList),
    Map(ConfigMap),
    Struct(ConfigStruct),
}

impl FromStrVariants for __ConfigTypeSerdeDerive {
    fn expected_variants() -> &'static [&'static str] {
        &["int", "str", "bool", "file-path", "folder-path"]
    }

    fn from_str(s: &str) -> Option<Self>
    where
        Self: Sized,
    {
        Some(match s {
            "int" => Self::Int(ConfigInt::default()),
            "str" => Self::Str(ConfigStr::default()),
            "bool" => Self::Bool(ConfigBool::default()),
            "file-path" => Self::FilePath(ConfigFilePath::default()),
            "folder-path" => Self::FolderPath(ConfigFolderPath::default()),
            _ => return None,
        })
    }
}

// other misc implementation details //

impl<'de> Deserialize<'de> for ConfigType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use __ConfigTypeSerdeDerive as Derived;
        string_or_struct::<'de, Derived, _>(deserializer).map(|value| match value {
            Derived::Int(config_int) => Self::Int(config_int),
            Derived::Str(config_str) => Self::Str(config_str),
            Derived::Bool(config_bool) => Self::Bool(config_bool),
            Derived::FilePath(config_file_path) => Self::FilePath(config_file_path),
            Derived::FolderPath(config_folder_path) => Self::FolderPath(config_folder_path),
            Derived::List(config_list) => Self::List(config_list),
            Derived::Map(config_map) => Self::Map(config_map),
            Derived::Struct(config_struct) => Self::Struct(config_struct),
        })
    }
}

/// [`FromStr`] that is just one of several possibilities.
///
/// The error type should be the possible variants.
trait FromStrVariants {
    fn expected_variants() -> &'static [&'static str];
    fn from_str(s: &str) -> Option<Self>
    where
        Self: Sized;
}

// https://serde.rs/string-or-struct.html
// slightly modified from requiring `FromStr<Err = Infallible>`
// to one of a selection of strings
fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStrVariants,
    D: Deserializer<'de>,
{
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStrVariants,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            match FromStrVariants::from_str(value) {
                Some(variant) => Ok(variant),
                None => Err(de::Error::unknown_variant(value, T::expected_variants())),
            }
        }

        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}

// defining this in a module so that i can use it above
mod macros {
    macro_rules! make_config_subtypes {
        (
            $(
                $(#[$inner_meta:meta])*
                pub struct $variant:ident {
                    $(
                        $field_vis:vis $field:ident : $field_ty:ty = $field_default:expr
                    ),*
                    $(,)?
                }
            )*
        ) => {
            $(
                $(#[$inner_meta])*
                #[derive(Debug, Deserialize, PartialEq)]
                #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(rename_all = "camelCase"))]
                #[serde(default, rename_all = "kebab-case")]
                pub struct $variant {
                    $( $field_vis $field : $field_ty ),*
                }

                impl Default for $variant {
                    fn default() -> Self {
                        Self {
                            $( $field: $field_default ),*
                        }
                    }
                }
            )*
        };
    }
    pub(super) use make_config_subtypes;
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use indexmap::IndexMap;

    use super::{
        ConfigInt, ConfigList, ConfigMap, ConfigSchema, ConfigStruct, ConfigType, PluginManifest,
    };

    #[test]
    fn full() -> Result<(), toml::de::Error> {
        let input = r#"
            name = "test"
            description = "my description"

            [schema.first-option]
            title = "first option"
            type = "int"
        "#;
        let output: PluginManifest = toml::from_str(input)?;
        assert_eq!(output, PluginManifest {
            name: "test".to_string(),
            description: Some("my description".to_string()),
            repository: None,
            authors: vec![],
            schema: IndexMap::from([("first-option".to_string(), ConfigSchema {
                title: "first option".to_string(),
                description: None,
                r#type: ConfigType::Int(ConfigInt::default())
            })])
        });

        Ok(())
    }

    #[test]
    fn int() {
        let input = r#"
            int = { min = 0 }
        "#;
        let output: ConfigType = toml::from_str(&input).unwrap();
        assert_eq!(
            output,
            ConfigType::Int(ConfigInt {
                min: 0,
                ..Default::default()
            })
        );
    }

    #[test]
    fn list() {
        let input = r#"
            title = "thing"
            type = { list = { item-type = "int", unique = true } }
        "#;
        let output: ConfigSchema = toml::from_str(input).unwrap();
        assert_eq!(output, ConfigSchema {
            title: "thing".to_string(),
            description: None,
            r#type: ConfigType::List(ConfigList {
                item_type: Box::new(ConfigType::Int(ConfigInt::default())),
                min_items: 0,
                unique: true,
            })
        });
    }

    #[test]
    fn open_plugin() {
        let input = r#"
            name = "Open"
            description = "Open URLs with a query"
            repository = "https://github.com/blorbb/comette-plugins"
            authors = ["blorbb"]

            [schema.urls]
            title = "List of URLs to show"
            type.map.value-type.struct.fields = { name = "str", url = "str" }
        "#;
        let output: PluginManifest = toml::from_str(input).unwrap();
        assert_eq!(output, PluginManifest {
            name: "Open".to_string(),
            description: Some("Open URLs with a query".to_string()),
            repository: Some("https://github.com/blorbb/comette-plugins".to_string()),
            authors: vec!["blorbb".to_string()],
            schema: IndexMap::from([("urls".to_string(), ConfigSchema {
                title: "List of URLs to show".to_string(),
                description: None,
                r#type: ConfigType::Map(ConfigMap {
                    value_type: Box::new(ConfigType::Struct(ConfigStruct {
                        fields: HashMap::from([
                            ("name".to_string(), ConfigType::Str(Default::default())),
                            ("url".to_string(), ConfigType::Str(Default::default()))
                        ])
                    })),
                    min_items: Default::default()
                })
            })])
        })
    }
}
