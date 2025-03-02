//! Generates an extension trait for `covey_plugin::ListItem`
//! for each of the commands in the manifest.

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use super::CratePaths;
use crate::{keyed_list::Identify, manifest::PluginManifest};

pub(super) fn generate_ext_trait(manifest: &PluginManifest, paths: &CratePaths) -> TokenStream {
    let covey_plugin = &paths.covey_plugin;

    let signatures: Vec<_> = manifest
        .commands
        .iter()
        .map(|key| {
            let method = Ident::new(&format!("on_{}", key.id().as_str().replace('-', "_")), Span::call_site());

            quote! {
                fn #method(
                    self,
                    callback: impl AsyncFn(#covey_plugin::Menu) -> #covey_plugin::Result<()> + ::core::marker::Send + ::core::marker::Sync + 'static
                ) -> Self
            }
        })
        .collect();

    let trait_def = quote! {
        pub trait CommandExt {
            #(#signatures;)*
        }
    };

    let menu_doclink = format!("[`Menu`]({}::Menu)", covey_plugin);
    let display_error_doclink = format!(
        "[`menu.display_error`]({}::Menu::display_error)",
        covey_plugin
    );
    let command_names = manifest.commands.iter().map(|item| item.id().as_str());
    let trait_impl = quote! {
        impl self::CommandExt for #covey_plugin::ListItem {
            #(
                /// Runs when this command is activated.
                ///
                /// The closure takes in a
                #[doc = #menu_doclink]
                /// as an argument. This can be used to perform some actions.
                ///
                /// If an [`Err`](::core::result::Result::Err) is returned,
                #[doc = #display_error_doclink]
                /// will be called on the error.
                #signatures {
                    let callback = ::std::sync::Arc::new(callback);
                    self.add_command(
                        #command_names,
                        ::std::sync::Arc::new(move |menu| ::std::boxed::Box::pin({
                            let callback = ::std::sync::Arc::clone(&callback);
                            async move {
                                if let ::core::result::Result::Err(e) = callback(::core::clone::Clone::clone(&menu)).await {
                                    menu.display_error(::std::format!("{e:#}"));
                                }
                            }
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
