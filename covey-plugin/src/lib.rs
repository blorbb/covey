pub mod manifest;
pub mod rank;

mod list;
use std::{
    path::PathBuf,
    sync::{LazyLock, OnceLock},
};

pub use list::{Icon, List, ListItem, ListStyle};
mod action;
pub use action::{Action, Actions};
mod input;
pub use input::{Input, SelectionRange};
mod plugin;
pub use plugin::Plugin;
mod server;
pub use server::run_server;
mod menu;
pub use menu::Menu;
mod poke;
pub mod spawn;
mod store;

pub use anyhow::{self, Result};

/// ID of this plugin.
///
/// This should be set to `env!("CARGO_PKG_NAME")` of the plugin, by
/// inputting string to [`main`].
pub static PLUGIN_ID: OnceLock<&'static str> = OnceLock::new();

/// Assigned directory of this plugin, where extra data can be stored.
///
/// This is <data-dir>/covey/plugins/<plugin-id>/. The directory should already
/// contain this plugin's binary (with the name of <plugin-id>) and
/// a `manifest.toml`.
///
/// [`covey_plugin`](crate) will also add an `activations.json` file. See
/// the [`rank`] module for more details.
///
/// This depends on the [`PLUGIN_ID`] static being set before first being called.
pub fn plugin_data_dir() -> &'static PathBuf {
    static DIR: LazyLock<PathBuf> = LazyLock::new(|| {
        dirs::data_dir()
            .expect("data dir should exist")
            .join("covey")
            .join("plugins")
            .join(
                PLUGIN_ID
                    .get()
                    .expect("plugin id should be initialised in main"),
            )
    });
    &*DIR
}

/// Clones variables into an async closure (by calling [`ToOwned::to_owned`]).
///
/// The closure will automatically be turned into a
/// `move || async move {...}` closure. Closure arguments are supported.
///
/// All variables will be cloned immediately for the closure to own
/// the variable. The variable will also be cloned every time the
/// closure is called, so that the async body can also own the variables.
///
/// # Examples
/// Expansion:
/// ```
/// # use covey_plugin::clone_async;
/// let thing = String::from("important info");
/// let foo = String::from("bar");
/// clone_async!(thing, foo, || {
///     println!("some {thing} from {foo}");
/// });
/// // Expands to:
/// // let thing = thing.to_owned();
/// // let foo = foo.to_owned();
/// // move || {
/// //     let thing = thing.to_owned();
/// //     let foo = foo.to_owned();
/// //     async move {
/// //         println!("some {thing} from {foo}");
/// //     }
/// // }
/// ```
///
/// This will most often be used in the context of adding callbacks to
/// list items.
/// ```ignore
/// # use covey_plugin::{clone_async, ListItem};
/// let thing = String::from("important info");
/// let foo = String::from("bar");
/// ListItem::new("title")
///     .on_activate(clone_async!(thing, || {
///         println!("some {thing}");
///         todo!()
///     }))
///     .on_hotkey_activate(clone_async!(foo, |hotkey| {
///         println!("foo {foo}! got hotkey {hotkey:?}");
///         todo!()
///     }));
/// ```
///
/// If you have a more complex expression that you want to clone, you can
/// bind it to an identifier using `ident = expr` syntax.
/// ```ignore
/// # use covey_plugin::{clone_async, ListItem};
/// let thing = String::from("important info");
/// let item = ListItem::new("get me out of here");
/// ListItem::new("i got u")
///     .on_activate(clone_async!(thing, title = item.title, || {
///         println!("got {title} out of there with {thing}");
///         todo!()
///     }));
/// ```
#[macro_export]
macro_rules! clone_async {
    // this isn't correctly matched with the below so need a special case
    // for empty closure args

    (
        $( $ident:ident $(= $expr:expr)? , )*
        || $($tt:tt)*
    ) => {
        {
            $(let $ident = ($crate::__clone_helper_choose_first!($($expr,)? $ident)).to_owned();)*
            async move || {
                // TODO: this extra clone isn't necessary.
                // with an async closure, the future can *borrow* from variables
                // captured (and owned) by the closure. however, this isn't clear
                // from the types and the compiler errors are sub-par right now.
                // this macro name also suggests that all variables should be owned.
                // This will always clone the captured variables every time this
                // closure is called, which is simpler to reason about and
                // this has little impact to performance anyways.
                $(let $ident = $ident.to_owned();)*
                { $($tt)* }
            }
        }
    };

    (
        $( $ident:ident $(= $expr:expr)? , )*
        | $($args:pat_param),* $(,)? | $($tt:tt)*
    ) => {
        {
            $(let $ident = ($crate::__clone_helper_choose_first!($($expr,)? $ident)).to_owned();)*
            async move | $($args),* | {
                $(let $ident = $ident.to_owned();)*
                { $($tt)* }
            }
        }
    };

}

/// Private implementation detail of [`clone_async!`].
#[doc(hidden)]
#[macro_export]
macro_rules! __clone_helper_choose_first {
    ($a:expr) => {
        $a
    };
    ($a:expr, $b: expr) => {
        $a
    };
}
