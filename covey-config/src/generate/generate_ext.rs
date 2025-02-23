//! Generates an extension trait for `covey_plugin::ListItem`
//! for each of the commands in the manifest.

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use super::CratePaths;
use crate::{keyed_list::Identify, manifest::PluginManifest};

pub(super) fn generate_ext_trait(manifest: &PluginManifest, paths: &CratePaths) -> TokenStream {
    let ret = &paths.command_return_ty;
    let ret_trait = &paths.command_return_trait;

    let signatures: Vec<_> = manifest
        .commands
        .iter()
        .map(|key| {
            let method = Ident::new(&format!("on_{}", key.id().as_str().replace('-', "_")), Span::call_site());

            quote! {
                fn #method<R>(
                    self,
                    callback: impl AsyncFn() -> #ret + ::core::marker::Send + ::core::marker::Sync + 'static
                ) -> Self
                where
                    R: #ret_trait
            }
        })
        .collect();

    let trait_def = quote! {
        pub trait CommandExt {
            #(#signatures;)*
        }
    };

    let ext_impl_ty = &paths.ext_impl_ty;
    let command_names = manifest.commands.iter().map(|item| item.id().as_str());
    let trait_impl = quote! {
        impl self::CommandExt for #ext_impl_ty {
            #(
                /// Runs when this command is activated.
                ///
                /// The closure can return any type that implements [`Into<Actions>`].
                /// This includes `impl IntoIterator<Item = Action>`, a single [`Action`],
                /// or an [`Input`].
                #signatures {
                    let callback = ::std::sync::Arc::new(callback);
                    self.add_command(
                        #command_names,
                        ::std::sync::Arc::new(move || ::std::boxed::Box::pin({
                            let callback = ::std::sync::Arc::clone(&callback);
                            async move { callback().await.map(::core::convert::Into::into) }
                        }))
                    )
                }
            )*
        }
    };

    quote! {
        #trait_def

        #trait_impl
    }
}
