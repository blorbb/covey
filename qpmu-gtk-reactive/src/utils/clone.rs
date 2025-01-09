#[expect(unused_macros, reason = "use eventually")]
macro_rules! clone {
    ($($ident:ident),* $(,)?) => {
        $(let $ident = $ident.to_owned();)*
    };
}

macro_rules! clone_scoped {
    ($ident:ident, $($tt:tt)*) => {
        {
            let $ident = $ident.to_owned();
            $crate::utils::clone::clone_scoped!($($tt)*)
        }
    };
    ($($tt:tt)*) => {
        $($tt)*
    }
}

#[expect(unused_imports, reason = "use eventually")]
pub(crate) use clone;
pub(crate) use clone_scoped;
