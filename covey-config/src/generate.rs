//! Private implementation details of the `include_manifest!` macro.

mod generate_ext;
mod generate_types;

use proc_macro2::TokenStream;
use quote::quote;

use crate::manifest::PluginManifest;

/// Type alias for a [`TokenStream`]. Just for better readability.
type Path = TokenStream;

pub fn include_manifest(
    s: &str,
    serde_path: Path,
    ext_impl_ty: Path,
    command_return_ty: Path,
    command_return_trait: Path,
) -> Result<TokenStream, toml::de::Error> {
    let paths = CratePaths {
        serde: serde_path,
        ext_impl_ty,
        command_return_ty,
        command_return_trait,
    };
    let manifest = PluginManifest::try_from_toml(s)?;

    let types = generate_types::generate_types(&manifest, &paths);
    let ext_trait = generate_ext::generate_ext_trait(&manifest, &paths);

    Ok(quote! {
        #types
        #ext_trait
    })
}

struct CratePaths {
    serde: Path,
    /// Path to `covey_plugin::ListItem`, to implement the extension trait.
    ext_impl_ty: Path,
    /// The return type of a command, [`Result<R>`].
    ///
    /// The generic `R` can be used, which will be [`CratePaths::command_return_trait`]
    command_return_ty: Path,
    /// Generic within [`CratePaths::command_return_ty`].
    command_return_trait: Path,
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
                #error::invalid_value(#serde::de::Unexpected::#variant, &#exp)
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
