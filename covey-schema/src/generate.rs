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
    covey_plugin_path: Path,
) -> Result<TokenStream, toml::de::Error> {
    let paths = CratePaths {
        serde: serde_path,
        covey_plugin: covey_plugin_path,
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
    covey_plugin: Path,
}

impl CratePaths {
    pub fn serde_error(&self) -> TokenStream {
        let serde = &self.serde;
        quote! { <D::Error as #serde::de::Error> }
    }

    #[expect(clippy::needless_pass_by_value, reason = "usually used with quote!()")]
    pub fn bail_invalid_value(&self, variant: TokenStream, exp: &str) -> TokenStream {
        let serde = &self.serde;
        let error = self.serde_error();
        quote! {
            return ::core::result::Result::Err(
                #error::invalid_value(#serde::de::Unexpected::#variant, &#exp)
            );
        }
    }

    #[expect(clippy::needless_pass_by_value, reason = "usually used with quote!()")]
    pub fn bail_invalid_length(&self, length: TokenStream, exp: &str) -> TokenStream {
        let error = self.serde_error();
        quote! {
            return ::core::result::Result::Err(
                #error::invalid_length(#length, &#exp)
            );
        }
    }
}
