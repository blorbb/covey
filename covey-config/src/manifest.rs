//! Types for the plugin manifest.

use std::{
    collections::HashMap,
    fmt::{self, Debug},
    marker::PhantomData,
};

use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, MapAccess, Visitor},
};

use crate::{
    hotkey::Hotkey,
    keyed_list::{Id, Identify, KeyedList},
};

/// A manifest for a single plugin.
///
/// This should be a TOML file stored in
/// `data directory/covey/plugins/<plugin>/manifest.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
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
    #[serde(default)]
    pub schema: KeyedList<PluginConfigSchema>,
    /// All commands that the user can run on some list item.
    ///
    /// The key an ID for the command, which is used when calling commands
    /// on the plugin.
    ///
    /// Several commands can have the same hotkey, but the commands that
    /// a single list item has should have different hotkeys.
    #[serde(default = "default_commands")]
    pub commands: KeyedList<Command>,
}

impl PluginManifest {
    pub fn try_from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub struct Command {
    pub id: Id,
    pub title: String,
    pub description: Option<String>,
    pub default_hotkey: Option<Hotkey>,
}

impl Identify for Command {
    fn id(&self) -> &Id {
        &self.id
    }
}

fn default_commands() -> KeyedList<Command> {
    KeyedList::new(vec![
        Command {
            id: Id::new("activate"),
            title: String::from("Activate"),
            description: None,
            default_hotkey: Some("enter".parse().expect("enter should be a hotkey")),
        },
        Command {
            id: Id::new("complete"),
            title: String::from("Complete"),
            description: None,
            default_hotkey: Some("tab".parse().expect("tab should be a hotkey")),
        },
        Command {
            id: Id::new("alt-activate"),
            title: String::from("Alt activate"),
            description: None,
            default_hotkey: Some("alt+enter".parse().expect("alt+enter should be a hotkey")),
        },
    ])
    .expect("ids are unique")
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[non_exhaustive]
pub struct PluginConfigSchema {
    pub id: Id,
    pub title: String,
    pub description: Option<String>,
    pub r#type: SchemaType,
}

impl Identify for PluginConfigSchema {
    fn id(&self) -> &Id {
        &self.id
    }
}

/// TODO: better docs
///
/// If there is no default, then this type will be *required*.
#[derive(Debug, PartialEq, Clone, Serialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub enum SchemaType {
    Int(SchemaInt),
    Text(SchemaText),
    Bool(SchemaBool),
    FilePath(SchemaFilePath),
    FolderPath(SchemaFolderPath),
    Selection(SchemaSelection),
    List(SchemaList),
    Map(SchemaMap),
    Struct(SchemaStruct),
}

// the below structs can't use the macro because they have extra
// required fields.
// all structs should have the same serde meta tag.

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub struct SchemaList {
    pub item_type: Box<SchemaType>,
    #[serde(default)]
    pub min_items: u32,
    /// Whether all items in the list must be unique.
    #[serde(default)]
    pub unique: bool,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
/// A map from any string to a specified value.
pub struct SchemaMap {
    pub value_type: Box<SchemaType>,
    #[serde(default)]
    pub min_items: u32,
}

/// A map with specific key-value pairs.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub struct SchemaStruct {
    pub fields: HashMap<String, SchemaType>,
}

/// A selection of one of multiple strings.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub struct SchemaSelection {
    pub allowed_values: Vec<String>,
    #[serde(default)]
    pub default: Option<String>,
}

macros::make_config_subtypes! {
    pub struct SchemaInt {
        pub min: i32 = i32::MIN,
        pub max: i32 = i32::MAX,
        pub default: Option<i32> = None,
    }
    pub struct SchemaText {
        pub min_length: u32 = u32::MIN,
        pub max_length: u32 = u32::MAX,
        pub default: Option<String> = None,
    }
    pub struct SchemaBool {
        pub default: Option<bool> = None,
    }
    pub struct SchemaFilePath {
        pub extension: Option<Vec<String>> = None,
        pub default: Option<String> = None,
    }
    pub struct SchemaFolderPath {
        pub default: Option<String> = None,
    }
}

/// Equivalent to [`SchemaType`] but with a derived deserialisation
/// implementation.
///
/// This is needed to avoid adding `#[deserialize_with = "string_or_struct"]`
/// on every instance of [`SchemaType`], and to be used in nested types like
/// a [`HashMap<_, SchemaType>`].
///
/// [`SchemaType`] has a manual deserialisation implementation that uses
/// the deserialisation of this.
///
/// [`SchemaType`] isn't a struct wrapper around this so that users can match
/// on it's variants.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum __SchemaTypeSerdeDerive {
    Int(SchemaInt),
    Text(SchemaText),
    Bool(SchemaBool),
    FilePath(SchemaFilePath),
    FolderPath(SchemaFolderPath),
    Selection(SchemaSelection),
    List(SchemaList),
    Map(SchemaMap),
    Struct(SchemaStruct),
}

impl FromStrVariants for __SchemaTypeSerdeDerive {
    fn expected_variants() -> &'static [&'static str] {
        &["int", "text", "bool", "file-path", "folder-path"]
    }

    fn from_str(s: &str) -> Option<Self>
    where
        Self: Sized,
    {
        Some(match s {
            "int" => Self::Int(SchemaInt::default()),
            "text" => Self::Text(SchemaText::default()),
            "bool" => Self::Bool(SchemaBool::default()),
            "file-path" => Self::FilePath(SchemaFilePath::default()),
            "folder-path" => Self::FolderPath(SchemaFolderPath::default()),
            _ => return None,
        })
    }
}

// other misc implementation details //

impl<'de> Deserialize<'de> for SchemaType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use __SchemaTypeSerdeDerive as Derived;
        string_or_struct::<'de, Derived, _>(deserializer).map(|value| match value {
            Derived::Int(config_int) => Self::Int(config_int),
            Derived::Text(config_str) => Self::Text(config_str),
            Derived::Bool(config_bool) => Self::Bool(config_bool),
            Derived::FilePath(config_file_path) => Self::FilePath(config_file_path),
            Derived::FolderPath(config_folder_path) => Self::FolderPath(config_folder_path),
            Derived::Selection(selection) => Self::Selection(selection),
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
                #[derive(Debug, Deserialize, PartialEq, Clone, Serialize)]
                #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
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

    use super::{
        PluginConfigSchema, PluginManifest, SchemaInt, SchemaList, SchemaMap, SchemaStruct,
        SchemaType,
    };
    use crate::{
        keyed_list::{Id, KeyedList},
        manifest::{SchemaSelection, default_commands},
    };

    #[test]
    fn full() -> Result<(), toml::de::Error> {
        let input = r#"
            name = "test"
            description = "my description"

            [[schema]]
            id = "first-option"
            title = "first option"
            type = "int"
        "#;
        let output: PluginManifest = toml::from_str(input)?;
        assert_eq!(
            output,
            PluginManifest {
                name: "test".to_string(),
                description: Some("my description".to_string()),
                repository: None,
                authors: vec![],
                schema: KeyedList::new([PluginConfigSchema {
                    id: Id::new("first-option"),
                    title: "first option".to_string(),
                    description: None,
                    r#type: SchemaType::Int(SchemaInt::default())
                }])
                .unwrap(),
                commands: default_commands(),
            }
        );

        Ok(())
    }

    #[test]
    fn int() {
        let input = r#"
            int = { min = 0 }
        "#;
        let output: SchemaType = toml::from_str(&input).unwrap();
        assert_eq!(
            output,
            SchemaType::Int(SchemaInt {
                min: 0,
                ..Default::default()
            })
        );
    }

    #[test]
    fn list() {
        let input = r#"
            id = "thing-id"
            title = "thing"
            type = { list = { item-type = "int", unique = true } }
        "#;
        let output: PluginConfigSchema = toml::from_str(input).unwrap();
        assert_eq!(
            output,
            PluginConfigSchema {
                id: Id::new("thing-id"),
                title: "thing".to_string(),
                description: None,
                r#type: SchemaType::List(SchemaList {
                    item_type: Box::new(SchemaType::Int(SchemaInt::default())),
                    min_items: 0,
                    unique: true,
                })
            }
        );
    }

    #[test]
    fn open_plugin() {
        let input = r#"
            name = "Open"
            description = "Open URLs with a query"
            repository = "https://github.com/blorbb/covey-plugins"
            authors = ["blorbb"]

            [[schema]]
            id = "urls"
            title = "List of URLs to show"
            type.map.value-type.struct.fields = { name = "text", url = "text" }
        "#;
        let output: PluginManifest = toml::from_str(input).unwrap();
        assert_eq!(
            output,
            PluginManifest {
                name: "Open".to_string(),
                description: Some("Open URLs with a query".to_string()),
                repository: Some("https://github.com/blorbb/covey-plugins".to_string()),
                authors: vec!["blorbb".to_string()],
                schema: KeyedList::new([PluginConfigSchema {
                    id: Id::new("urls"),
                    title: "List of URLs to show".to_string(),
                    description: None,
                    r#type: SchemaType::Map(SchemaMap {
                        value_type: Box::new(SchemaType::Struct(SchemaStruct {
                            fields: HashMap::from([
                                ("name".to_string(), SchemaType::Text(Default::default())),
                                ("url".to_string(), SchemaType::Text(Default::default()))
                            ])
                        })),
                        min_items: Default::default()
                    })
                }])
                .unwrap(),
                commands: default_commands(),
            }
        )
    }

    #[test]
    fn selection() {
        let input = r#"
            selection.allowed-values = ["some-thing", "another-thing", "and-yet-another"]
            selection.default = "some-thing"
        "#;
        let output: SchemaType = toml::from_str(&input).unwrap();
        assert_eq!(
            output,
            SchemaType::Selection(SchemaSelection {
                allowed_values: vec![
                    "some-thing".to_string(),
                    "another-thing".to_string(),
                    "and-yet-another".to_string()
                ],
                default: Some("some-thing".to_string())
            })
        )
    }
}
