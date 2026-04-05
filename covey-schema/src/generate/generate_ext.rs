//! Generates an extension trait for `covey_plugin::ListItem`
//! for each of the commands in the manifest.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::CratePaths;
use crate::{id::StringId as _, keyed_list::Identify, manifest::PluginManifest};

pub(super) fn generate_ext_trait(manifest: &PluginManifest, paths: &CratePaths) -> TokenStream {
    let covey_plugin = &paths.covey_plugin;

    let command_ids: Vec<_> = manifest
        .commands
        .iter()
        .map(|item| item.id().as_str())
        .collect();
    let signatures: Vec<_> = command_ids.iter()
        .map(|command| {
            let method = format_ident!("on_{}", command.replace('-', "_"));

            quote! {
                fn #method(
                    self,
                    callback: impl AsyncFn(#covey_plugin::Menu) -> #covey_plugin::Result<()> + ::core::marker::Send + ::core::marker::Sync + 'static
                ) -> Self
            }
        })
        .collect();

    let menu_doclink = format!("[`Menu`]({covey_plugin}::Menu)");
    let display_error_doclink =
        format!("[`menu.display_error`]({covey_plugin}::Menu::display_error)");

    let trait_def = quote! {
        pub trait CommandExt {
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
                #signatures;
            )*
        }
    };

    let list_item_impl = quote! {
        impl self::CommandExt for #covey_plugin::ListItem {
            #(
                #signatures {
                    self.add_command(
                        #command_ids,
                        callback
                    )
                }
            )*
        }
    };

    let list_impl = quote! {
        impl self::CommandExt for #covey_plugin::List {
            #(
                #signatures {
                    self.add_command(
                        #command_ids,
                        callback
                    )
                }
            )*
        }
    };

    quote! {
        #trait_def

        #list_item_impl
        #list_impl
    }
}
