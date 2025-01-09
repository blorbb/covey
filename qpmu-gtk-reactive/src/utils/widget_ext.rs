//! Extension traits for widgets and their builders.
//!
//! Blanket implementations are difficult because Rust doesn't know that the
//! builder type will not implement [`IsA<gtk::Widget>`].
//!
//! This is worked around with the [`IntoWidget`] trait, which is manually
//! implemented for each [`gtk::Widget`] and their builder. The [`WidgetBuilder`]
//! trait is implemented for all widget builders, only providing a
//! [`build`](WidgetBuilder::build). This also needs to be separate trait
//! that is not implemented for [`gtk::Widget`]s to avoid conflicting
//! implementations.
//!
//! Extension traits are then implemented for each [`gtk::Widget`] manually,
//! and a blanket implementation for the [`WidgetBuilder`] can be created.

use gtk::prelude::{BoxExt as _, IsA};

use super::stores::WidgetRef;

// traits for every widget //

/// Any type that can be transformed into a widget.
///
/// A blanket implementation for [`gtk::Widget`]s doesn't exist as it could
/// conflict with builder implementations. This should be manually implemented
/// for every gtk widget using [`impl_widget!`].
pub trait IntoWidget: Sized {
    type Widget: IsA<gtk::Widget>;
}

/// A builder that builds into a widget.
trait WidgetBuilder: IntoWidget {
    fn build(self) -> Self::Widget;
}

/// Adds a `widget_ref` builder-like method for all widgets and their builders.
pub trait WidgetSetRef: IntoWidget {
    fn widget_ref(self, r: WidgetRef<Self::Widget>) -> Self::Widget;
}

macro_rules! impl_widget {
    ($($widget:ident)*) => {
        $(paste::paste! {
            impl IntoWidget for gtk::builders::[<$widget Builder>] {
                type Widget = gtk::$widget;
            }

            impl IntoWidget for gtk::$widget {
                type Widget = Self;
            }

            impl WidgetBuilder for gtk::builders::[<$widget Builder>] {
                fn build(self) -> Self::Widget {
                    self.build()
                }
            }

            impl WidgetSetRef for gtk::$widget {
                fn widget_ref(self, r: WidgetRef<Self::Widget>) -> Self::Widget {
                    r.set(self.clone());
                    self
                }
            }
        })*

    };
}

impl_widget!(Box ScrolledWindow Entry FlowBox FlowBoxChild ApplicationWindow);

// widget-specific extension traits //

/// Builder-like method for widgets which can hold multiple children.
pub trait WidgetAddChild: IntoWidget {
    fn child(self, child: &impl IsA<gtk::Widget>) -> Self::Widget;
}

impl WidgetAddChild for gtk::Box {
    fn child(self, child: &impl IsA<gtk::Widget>) -> Self::Widget {
        self.append(child);
        self
    }
}

// blanket builder implementations //

// how to read the trait bound
// T is a builder that builds a widget U, and U implements the same
// trait but it outputs itself (U::Widget = T::Widget = U)

impl<T> WidgetSetRef for T
where
    T: WidgetBuilder<Widget: WidgetSetRef<Widget = Self::Widget>>,
{
    fn widget_ref(self, r: WidgetRef<Self::Widget>) -> Self::Widget {
        self.build().widget_ref(r)
    }
}

impl<T> WidgetAddChild for T
where
    T: WidgetBuilder<Widget: WidgetAddChild<Widget = Self::Widget>>,
{
    fn child(self, child: &impl IsA<gtk::Widget>) -> Self::Widget {
        self.build().child(child)
    }
}
