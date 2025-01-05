use gtk::{
    builders::BoxBuilder,
    prelude::{BoxExt, IsA},
    Widget,
};

use crate::reactive::WidgetRef;

/// Builder-like method for widgets which can hold multiple children.
pub trait ManyChildren: Sized {
    type Output;

    fn child(self, child: &impl IsA<Widget>) -> Self::Output;
}

impl ManyChildren for gtk::Box {
    type Output = Self;

    fn child(self, child: &impl IsA<Widget>) -> Self {
        self.append(child);
        self
    }
}

impl ManyChildren for BoxBuilder {
    type Output = gtk::Box;

    fn child(self, child: &impl IsA<Widget>) -> Self::Output {
        self.build().child(child)
    }
}

/// Adds a `widget_ref` builder-like method for all widgets and their builders.
pub trait SetWidgetRef: Sized {
    type Output;

    fn widget_ref(self, r: WidgetRef<Self::Output>) -> Self::Output;
}

macro_rules! impl_ref {
    ($widget:ident) => {
        impl SetWidgetRef for gtk::$widget {
            type Output = Self;
            fn widget_ref(self, r: WidgetRef<Self::Output>) -> Self::Output {
                r.set(self.clone());
                self
            }
        }

        paste::paste! {
            impl SetWidgetRef for gtk::builders::[<$widget Builder>] {
                type Output = gtk::$widget;
                fn widget_ref(self, r: WidgetRef<Self::Output>) -> Self::Output {
                    self.build().widget_ref(r)
                }
            }
        }
    };
}

macro_rules! impl_refs {
    ($($widgets:ident)*) => {
        $(impl_ref!($widgets);)*
    };
}

impl_refs!(Box ScrolledWindow Entry FlowBox FlowBoxChild ApplicationWindow);

#[macro_export]
macro_rules! clone {
    ($($ident:ident),* $(,)?) => {
        $(let $ident = $ident.to_owned();)*
    };
}

#[macro_export]
macro_rules! clone_scoped {
    ($ident:ident, $($tt:tt)*) => {
        {
            let $ident = $ident.to_owned();
            $crate::clone_scoped!($($tt)*)
        }
    };
    ($($tt:tt)*) => {
        $($tt)*
    }
}
