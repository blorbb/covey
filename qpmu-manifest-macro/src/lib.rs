use proc_macro_error2::{abort, proc_macro_error};
use syn::{LitStr, parse_quote};

#[proc_macro]
#[proc_macro_error]
pub fn generate_config(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let str = syn::parse_macro_input!(input as LitStr);

    qpmu_manifest::generate::generate_config(
        &str.value(),
        parse_quote!(::serde),
        parse_quote!(::toml),
    )
    .unwrap_or_else(|e| abort!(str.span(), "invalid manifest toml: {}", e))
    .into()
}
