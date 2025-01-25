//! Generates an extension trait for `covey_plugin::ListItem`
//! for each of the commands in the manifest.

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use super::CratePaths;
use crate::PluginManifest;

pub(super) fn generate_ext_trait(manifest: &PluginManifest, paths: &CratePaths) -> TokenStream {
    let ret = &paths.command_return_ty;

    let signatures: Vec<_> = manifest
        .commands
        .keys()
        .map(|key| {
            let method = Ident::new(&format!("on_{}", key.replace('-', "_")), Span::call_site());

            quote! {
                fn #method<Fut>(
                    self,
                    callback: impl Fn() -> Fut + ::core::marker::Send + ::core::marker::Sync + 'static
                ) -> Self
                where
                    Fut: ::core::future::Future<Output = #ret>
                        + ::core::marker::Send + ::core::marker::Sync + 'static
            }
        })
        .collect();

    let trait_def = quote! {
        pub trait CommandExt {
            #(#signatures;)*
        }
    };

    let ext_impl_ty = &paths.ext_impl_ty;
    let command_names = manifest.commands.keys();
    let trait_impl = quote! {
        impl self::CommandExt for #ext_impl_ty {
            #(#signatures {
                self.add_command(
                    #command_names,
                    ::std::sync::Arc::new(move || ::std::boxed::Box::pin(callback()))
                )
            })*
        }
    };

    return quote! {
        #trait_def

        #trait_impl
    };
}
