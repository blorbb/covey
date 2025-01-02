use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::ext::IdentExt;

use crate::{
    ConfigBool, ConfigFilePath, ConfigFolderPath, ConfigInt, ConfigList, ConfigMap, ConfigStr,
    ConfigStruct, ConfigType, PluginManifest,
};

/// Type alias for a [`TokenStream`]. Just for better readability.
type Path = TokenStream;

pub fn generate_config(s: &str, serde_path: Path) -> Result<TokenStream, toml::de::Error> {
    Ok(generate_types(
        PluginManifest::try_from_toml(s)?,
        serde_path,
    ))
}

fn generate_types(manifest: PluginManifest, serde_path: Path) -> TokenStream {
    let field = FieldType::from_struct(
        ConfigStruct {
            fields: manifest
                .schema
                .into_iter()
                .map(|(k, v)| (k, v.r#type))
                .collect(),
        },
        &CratePaths { serde: serde_path },
        &Ident::new("config", Span::call_site()),
    );

    field.extras
}

struct CratePaths {
    serde: Path,
}

impl CratePaths {
    pub fn serde_error(&self) -> TokenStream {
        let serde = &self.serde;
        quote! { <D::Error as #serde::de::Error> }
    }

    pub fn bail_invalid_value(&self, variant: TokenStream, exp: &str) -> TokenStream {
        let serde = &self.serde;
        let error = self.serde_error();
        quote! {
            return ::core::result::Result::Err(
                #error::invalid_value(#serde::de::Unexpected::#variant, #exp)
            );
        }
    }

    pub fn bail_invalid_length(&self, length: TokenStream, exp: &str) -> TokenStream {
        let error = self.serde_error();
        quote! {
            return ::core::result::Result::Err(
                #error::invalid_length(#length, &#exp)
            );
        }
    }
}

struct FieldType {
    /// Full type path of this field.
    ///
    /// Examples:
    /// - `::core::primitive::i64`
    /// - `::std::collections::HashMap<::std::string::String, self::MapValue>`
    type_path: TypePath,
    /// Function body of the deserializer.
    ///
    /// These variables will be in scope.
    /// - `D: Deserializer<'de>`
    /// - `deserializer: D`
    /// - `value: &...`
    ///
    /// Should be used with `#[serde(deserialize_with = "path")]`.
    ///
    /// Examples:
    /// ```ignore
    /// fn deserializer_name<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i64, D::Error> {
    ///     let value: i64 = ::serde::Deserialize::deserialize(deserializer)?;
    ///     {
    ///         let value = &value;
    ///         // start included section
    ///         if value < 0 {
    ///             // ...
    ///         }
    ///         // end included section
    ///     }
    ///     Ok(value)
    /// }
    /// ```
    validator: TokenStream,
    /// Default annotation to add, if any.
    default: TypeDefault,
    /// Extra definitions to add to the module.
    extras: TokenStream,
}

enum TypeDefault {
    /// No default value.
    Required,
    /// Use the `#[serde(default)]` annotation.
    ///
    /// This should only be used for collections (hash map and vec).
    DefaultTrait,
    /// Use the `#[serde(default = "path")]` annotation.
    ///
    /// The token stream is the function body of the default function.
    Custom(TokenStream),
}

impl<T: ToTokens> From<Option<T>> for TypeDefault {
    /// Converts a [`Some`] to [`TypeDefault::Custom`]
    /// and [`None`] to [`TypeDefault::Required`].
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Self::Custom(value.into_token_stream()),
            None => Self::Required,
        }
    }
}

impl FieldType {
    fn new(config: ConfigType, paths: &CratePaths, parent_key: &Ident) -> Self {
        match config {
            ConfigType::Int(int) => Self::from_int(int, paths),
            ConfigType::Str(str) => Self::from_str(str, paths),
            ConfigType::Bool(bool) => Self::from_bool(bool, paths),
            ConfigType::FilePath(file) => Self::from_file_path(file, paths),
            ConfigType::FolderPath(folder) => Self::from_folder_path(folder, paths),
            ConfigType::List(list) => Self::from_list(list, paths, parent_key),
            ConfigType::Map(map) => Self::from_map(map, paths, parent_key),
            ConfigType::Struct(st) => Self::from_struct(st, paths, parent_key),
        }
    }

    fn from_int(ConfigInt { min, max, default }: ConfigInt, paths: &CratePaths) -> Self {
        let min_error =
            paths.bail_invalid_value(quote!(Signed), &format!("value to be at least {min}"));
        let max_error =
            paths.bail_invalid_value(quote!(Signed), &format!("value to be at most {max}"));

        Self {
            type_path: TypePath::absolute(quote! { ::core::primitive::i64 }),
            validator: quote! {
                if value < #min { #min_error }
                if value > #max { #max_error }
            },
            default: TypeDefault::from(default),
            extras: TokenStream::new(),
        }
    }

    fn from_str(
        ConfigStr {
            min_length,
            max_length,
            default,
        }: ConfigStr,
        paths: &CratePaths,
    ) -> Self {
        let min_error = paths.bail_invalid_length(
            quote!(value.len()),
            &format!("length to be at least {min_length}"),
        );
        let max_error = paths.bail_invalid_length(
            quote!(value.len()),
            &format!("length to be at most {max_length}"),
        );

        Self {
            type_path: TypePath::absolute(quote! { ::std::string::String }),
            validator: quote! {
                if (value.len() as u64) < #min_length { #min_error }
                if (value.len() as u64) > #max_length { #max_error }
            },
            default: TypeDefault::from(default),
            extras: TokenStream::new(),
        }
    }

    fn from_bool(ConfigBool { default }: ConfigBool, _paths: &CratePaths) -> Self {
        Self {
            type_path: TypePath::absolute(quote! { ::core::primitive::bool }),
            validator: quote! {},
            default: TypeDefault::from(default),
            extras: TokenStream::new(),
        }
    }

    /// Does not check that the path is actually a valid file that exists.
    fn from_file_path(
        ConfigFilePath { extension, default }: ConfigFilePath,
        paths: &CratePaths,
    ) -> Self {
        let extension_check = extension.map(|exts| {
            let ext_error = paths.bail_invalid_value(
                quote!(Str(&value.to_string_lossy())),
                &format!("path to have an extension of one of {exts:?}"),
            );

            quote! {
                let ::core::option::Option::Some(value_ext) = value.extension() else {
                    #ext_error
                };
                if [#(#exts),*].into_iter().all(|ext| ext != value_ext) {
                    #ext_error
                }
            }
        });

        Self {
            type_path: TypePath::absolute(quote! { ::std::path::PathBuf }),
            validator: extension_check.unwrap_or_default(),
            default: TypeDefault::from(default),
            extras: TokenStream::new(),
        }
    }

    fn from_folder_path(
        ConfigFolderPath { default }: ConfigFolderPath,
        _paths: &CratePaths,
    ) -> Self {
        Self {
            type_path: TypePath::absolute(quote! { ::std::path::PathBuf }),
            validator: quote! {},
            default: TypeDefault::from(default),
            extras: TokenStream::new(),
        }
    }

    fn from_list(
        ConfigList {
            item_type,
            min_items,
            unique,
        }: ConfigList,
        paths: &CratePaths,
        parent_key: &Ident,
    ) -> Self {
        let FieldType {
            type_path: inner_type,
            validator: inner_validator,
            default: _,
            extras,
        } = Self::new(*item_type, paths, &format_ident!("{parent_key}_item"));

        let length_error = paths.bail_invalid_length(
            quote!(value.len()),
            &format!("list to have at least {min_items} elements"),
        );

        let unique_check = unique.then(|| -> TokenStream {
            let unique_error = paths.bail_invalid_value(
                quote!(Other(&::std::format!("{v:?}"))),
                "list to have no duplicates",
            );

            todo!("uniqueness validator");
        });

        Self {
            type_path: TypePath::absolute(quote! { ::std::vec::Vec })
                .with_generic(inner_type.clone()),
            validator: quote! {
                for value in value {
                    #inner_validator
                }

                if (value.len() as u64) < #min_items {
                    #length_error
                }

                #unique_check
            },
            default: TypeDefault::DefaultTrait,
            extras,
        }
    }

    fn from_map(
        ConfigMap {
            value_type,
            min_items,
        }: ConfigMap,
        paths: &CratePaths,
        parent_key: &Ident,
    ) -> Self {
        let FieldType {
            type_path: inner_type,
            validator: inner_validator,
            default: _,
            extras,
        } = Self::new(*value_type, paths, &format_ident!("{parent_key}_value"));

        let length_error = paths.bail_invalid_length(
            quote!(value.len()),
            &format!("map to have at least {min_items} entries"),
        );

        Self {
            type_path: TypePath::absolute(quote! { ::std::collections::HashMap })
                .with_generic(TypePath::absolute(quote! { ::std::string::String }))
                .with_generic(inner_type.clone()),
            validator: quote! {
                for value in value.values() {
                    #inner_validator
                }

                if (value.len() as u64) < #min_items {
                    #length_error
                }
            },
            default: TypeDefault::DefaultTrait,
            extras,
        }
    }

    /// `parent_key` is the key used to identify the struct, in snake_case.
    ///
    /// # Expansion
    ///
    /// The struct is added to [`Self::extras`] at the top level. Every field
    /// gets a module with the same name, containing extra types or the
    /// deserialize/default implementations.
    ///
    /// ```ignore
    /// pub struct SomeStruct {
    ///     #[serde(deserialize_with = self::something::deserialize, default = self::something::default)]
    ///     something: i64,
    ///     nested: self::nested::Nested,
    /// }
    ///
    /// pub mod something {
    ///     pub(super) fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i64, D::Error> {
    ///         // ...
    ///     }
    ///
    ///     pub(super) fn default() -> i64 {
    ///         // ...
    ///     }
    /// }
    ///
    /// pub mod nested {
    ///     pub struct Nested {}
    /// }
    /// ```
    fn from_struct(
        ConfigStruct { fields }: ConfigStruct,
        paths: &CratePaths,
        parent_key: &Ident,
    ) -> Self {
        let serde = &paths.serde;
        let struct_name = snake_to_upper_camel(&parent_key);
        let mut extras = TokenStream::new();
        let mut this_fields = TokenStream::new();

        for (key, ty) in fields {
            let field_name = Ident::new_raw(&key.replace('-', "_"), Span::call_site());

            let FieldType {
                type_path: mut field_type,
                validator: field_validator,
                default: field_default,
                extras: field_extras,
            } = Self::new(ty, paths, &field_name);

            let deserialize_annotation = {
                let path = quote! { self::#field_name::deserialize }.to_string();
                quote! { deserialize_with = #path, }
            };

            let (default_annotation, default_function) = match field_default {
                TypeDefault::Custom(inner) => {
                    let path = quote! { self::#field_name::default }.to_string();

                    (quote! { default = #path, }, quote! {
                        pub(super) fn default() -> #field_type {
                            #inner
                        }
                    })
                }
                TypeDefault::Required => (quote!(), quote!()),
                TypeDefault::DefaultTrait => (quote! { default, }, quote!()),
            };

            extras.extend(quote! {
                #[allow(unused, unused_comparisons)]
                pub mod #field_name {
                    pub(super) fn deserialize<'de, D: #serde::Deserializer<'de>>(
                        deserializer: D
                    ) -> ::core::result::Result<#field_type, D::Error> {
                        let value: #field_type = #serde::Deserialize::deserialize(deserializer)?;
                        {
                            let value = &value;
                            #field_validator
                        }
                        Ok(value)
                    }

                    #default_function

                    #field_extras
                }
            });

            field_type.nest_within(&field_name);
            this_fields.extend(quote! {
                #[serde(#deserialize_annotation #default_annotation)]
                pub #field_name: #field_type,
            });
        }

        Self {
            type_path: TypePath::relative(struct_name.to_token_stream()),
            validator: TokenStream::new(),
            default: TypeDefault::Required,
            extras: quote! {
                #[derive(
                    ::std::fmt::Debug,
                    ::std::cmp::PartialEq,
                    #serde::Deserialize,
                )]
                pub struct #struct_name {
                    #this_fields
                }

                #extras
            },
        }
    }
}

#[derive(Debug, Clone)]
struct TypePath {
    kind: TypePathKind,
    base: Path,
    generics: Vec<TypePath>,
}

impl TypePath {
    pub fn relative(base: Path) -> Self {
        Self {
            kind: TypePathKind::Relative,
            base,
            generics: vec![],
        }
    }

    pub fn absolute(base: Path) -> Self {
        Self {
            kind: TypePathKind::Absolute,
            base,
            generics: vec![],
        }
    }

    pub fn with_generic(mut self, generic: TypePath) -> Self {
        self.generics.push(generic);
        self
    }
}

#[derive(Debug, Clone, Copy)]
enum TypePathKind {
    Absolute,
    Relative,
}

impl TypePath {
    pub fn nest_within(&mut self, module: &Ident) {
        let Self {
            kind,
            base,
            generics,
        } = self;

        match kind {
            TypePathKind::Absolute => {}
            TypePathKind::Relative => *base = quote! { #module::#base },
        }
        generics.iter_mut().for_each(|ty| ty.nest_within(module));
    }
}

impl ToTokens for TypePath {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            kind,
            base,
            generics,
        } = self;
        match kind {
            TypePathKind::Absolute => tokens.extend(base.clone()),
            TypePathKind::Relative => tokens.extend(quote! { self::#base }),
        }

        if !generics.is_empty() {
            tokens.extend(quote! { < #(#generics),* > });
        }
    }
}

fn snake_to_upper_camel(snake: &Ident) -> Ident {
    let str = snake
        .unraw()
        .to_string()
        .split('_')
        .filter_map(|str| {
            // capitalise first character
            let mut chars = str.chars();
            Some(
                chars
                    .next()?
                    .to_uppercase()
                    .chain(chars)
                    .collect::<String>(),
            )
        })
        .collect::<String>();
    Ident::new_raw(&str, snake.span())
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::generate_config;

    #[test]
    fn expansion() {
        let input = r#"
        name = "Open"
        description = "Open URLs with a query"
        repository = "https://github.com/blorbb/qpmu-plugins"
        authors = ["blorbb"]

        [schema.urls]
        title = "List of URLs to show"

        [schema.urls.type]
        type-name = "map"
        value-type.type-name = "struct"
        value-type.fields = { name = "str", url = "str" }
        "#;

        let out = generate_config(&input, quote! {::serde}).unwrap();
        eprintln!("{out}");
    }
}
