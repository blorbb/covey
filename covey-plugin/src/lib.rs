pub mod manifest;
pub mod rank;
pub mod sql;

mod list;
pub use list::{Icon, List, ListItem, ListStyle};
mod action;
pub use action::Action;
mod input;
pub use input::{Input, SelectionRange};
mod plugin;
pub use plugin::Plugin;
mod hotkey;
pub use hotkey::{Hotkey, Key};
mod server;
pub use server::run_server as main;
mod plugin_lock;
mod store;

#[allow(clippy::pedantic)]
mod proto {
    tonic::include_proto!("plugin");
}

pub use anyhow::{self, Result};

/// Clones variables into an async closure (by calling [`ToOwned::to_owned`]).
///
/// The closure will automatically be turned into a
/// `move || async move {...}` closure. Closure arguments are supported.
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
/// ```
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
/// ```
/// # use covey_plugin::{clone_async, ListItem};
/// let thing = String::from("important info");
/// let item = ListItem::new("get me out of here");
/// ListItem::new("i got u")
///     .on_activate(clone_async!(thing, title = item.title, || {
///         println!("got {title} out of there with {thing}");
///         todo!()
///     }));
/// ```
///
/// Sometimes there are still some lifetime issues when cloning.
/// ```compile_fail
/// # use covey_plugin::{clone_async, ListItem};
/// // this fails to compile [E0597]: `thing` does not live long enough
/// {
///     let thing = String::from("really important info!!");
///     let temp = &thing;
///     ListItem::new("oh no")
///         .on_activate(clone_async!(temp, || {
///             todo!("temporary {temp}")
///         }))
/// };
/// ```
///
/// Add `#[double]` to clone the value into the closure as well
/// as the async block.
/// ```
/// # use covey_plugin::{clone_async, ListItem};
/// {
///     let thing = String::from("really important info!!");
///     let temp = &thing;
///     ListItem::new("oh no")
///         .on_activate(clone_async!(#[double] temp, || {
///             todo!("temporary {temp}")
///         }))
/// };
/// ```
#[macro_export]
macro_rules! clone_async {
    // this isn't correctly matched with the below so need a special case
    // for empty closure args
    (
        $( $(#[$double:ident])? $ident:ident $(= $expr:expr)? , )*
        || $($tt:tt)*
    ) => {
        {
            $(
                $crate::__clone_helper!($($double)? @ $ident $(, $expr)?);
            )*
            move || {
                $(let $ident = ($ident).to_owned();)*
                async move {$($tt)*}
            }
        }
    };

    (
        $( $(#[$double:ident])? $ident:ident $(= $expr:expr)? , )*
        | $($args:pat_param),* $(,)? | $($tt:tt)*
    ) => {
        {
            $(
                $crate::__clone_helper!($($double)? @ $ident $(, $expr)?);
            )*
            move | $($args),* | {
                $(let $ident = ($ident).to_owned();)*
                async move {$($tt)*}
            }
        }
    };
}

/// Private implementation detail of [`clone_async!`].
#[doc(hidden)]
#[macro_export]
macro_rules! __clone_helper {
    // clone either the ident given, or the expr if there exists one.
    // first clone: if `double` is present, actually do a clone.
    // otherwise a no-op.
    (double @ $ident:ident) => {
        let $ident = ($ident).to_owned();
    };
    (@ $ident:ident) => {};

    (double @ $ident:ident, $expr:expr) => {
        let $ident = ($expr).to_owned();
    };
    (@ $ident:ident, $expr:expr) => {
        // let ident = expr, easier for later, only need to handle cloning idents
        let $ident = $expr;
    };
}
