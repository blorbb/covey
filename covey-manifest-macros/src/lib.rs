use proc_macro_error2::{abort, proc_macro_error};
use proc_macro2::Span;
use quote::ToTokens;
use syn::{LitStr, Token, parse::Parse, parse_quote, punctuated::Punctuated};

#[proc_macro]
#[proc_macro_error]
pub fn include_manifest(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as Input);

    covey_schema::generate::include_manifest(
        &input.format.evaluate(),
        input.serde_path.into_token_stream(),
        input.ext_impl_ty.into_token_stream(),
        input.command_return_ty.into_token_stream(),
        input.command_return_trait.into_token_stream(),
    )
    .unwrap_or_else(|e| abort!(input.format.span(), "invalid manifest toml: {}", e))
    .into()
}

struct Input {
    serde_path: syn::Path,
    ext_impl_ty: syn::Type,
    command_return_ty: syn::Type,
    command_return_trait: syn::TraitBound,
    format: InputFormat,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut serde_path = None;
        let mut ext_impl_ty = None;
        let mut command_return_ty = None;
        let mut command_return_trait = None;
        let mut fmt = None;

        let entries = Punctuated::<Entry, Token![,]>::parse_terminated(input)?;
        for entry in entries {
            match entry {
                Entry::SerdePath(key, path) => {
                    if serde_path.replace(path).is_some() {
                        abort!(key, "`serde_path` must be defined once")
                    }
                }
                Entry::ExtImplTy(key, path) => {
                    if ext_impl_ty.replace(path).is_some() {
                        abort!(key, "`ext_impl_ty` must be defined once")
                    }
                }
                Entry::CommandReturnTy(key, path) => {
                    if command_return_ty.replace(path).is_some() {
                        abort!(key, "`command_return_ty` must be defined once")
                    }
                }
                Entry::CommandReturnTrait(key, path) => {
                    if command_return_trait.replace(path).is_some() {
                        abort!(key, "`command_return_trait` must be defined once")
                    }
                }
                Entry::Inline(key, str) => {
                    if fmt.replace(InputFormat::Inline(str)).is_some() {
                        abort!(key, "exactly one of `inline` or `file` must be defined");
                    }
                }
                Entry::File(key, str) => {
                    if fmt.replace(InputFormat::File(str)).is_some() {
                        abort!(key, "exactly one of `inline` or `file` must be defined");
                    }
                }
            }
        }

        Ok(Self {
            serde_path: serde_path.unwrap_or(parse_quote!(::serde)),
            ext_impl_ty: ext_impl_ty
                .unwrap_or_else(|| abort!(input.span(), "`ext_impl_ty` must be defined")),
            command_return_ty: command_return_ty
                .unwrap_or_else(|| abort!(input.span(), "`command_return_ty` must be defined")),
            command_return_trait: command_return_trait
                .unwrap_or_else(|| abort!(input.span(), "`command_return_trait` must be defined")),
            format: fmt.unwrap_or_else(|| {
                abort!(
                    input.span(),
                    "exactly one of `inline` or `file` must be defined"
                )
            }),
        })
    }
}

enum InputFormat {
    File(LitStr),
    Inline(LitStr),
}

impl InputFormat {
    fn evaluate(&self) -> String {
        match self {
            Self::Inline(str) => str.value(),
            Self::File(file) => {
                let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
                    .unwrap_or_else(|e| abort!(file, "could not read `CARGO_MANIFEST_DIR`: {}", e));
                let path = std::path::Path::new(&manifest_dir)
                    .join(file.value())
                    .canonicalize()
                    .unwrap_or_else(|e| abort!(file, "failed to canonicalize path: {}", e));

                std::fs::read_to_string(path)
                    .unwrap_or_else(|e| abort!(file, "could not read file: {}", e))
            }
        }
    }

    fn span(&self) -> Span {
        match self {
            Self::File(lit_str) | Self::Inline(lit_str) => lit_str.span(),
        }
    }
}

enum Entry {
    SerdePath(kw::serde_path, syn::Path),
    ExtImplTy(kw::ext_impl_ty, syn::Type),
    CommandReturnTy(kw::command_return_ty, syn::Type),
    CommandReturnTrait(kw::command_return_trait, syn::TraitBound),
    Inline(kw::inline, syn::LitStr),
    File(kw::file, syn::LitStr),
}

impl Parse for Entry {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::serde_path) {
            let key = input.parse::<kw::serde_path>()?;
            input.parse::<Token![=]>()?;
            Ok(Self::SerdePath(key, input.parse::<syn::Path>()?))
        } else if lookahead.peek(kw::ext_impl_ty) {
            let key = input.parse::<kw::ext_impl_ty>()?;
            input.parse::<Token![=]>()?;
            Ok(Self::ExtImplTy(key, input.parse::<syn::Type>()?))
        } else if lookahead.peek(kw::command_return_ty) {
            let key = input.parse::<kw::command_return_ty>()?;
            input.parse::<Token![=]>()?;
            Ok(Self::CommandReturnTy(key, input.parse::<syn::Type>()?))
        } else if lookahead.peek(kw::command_return_trait) {
            let key = input.parse::<kw::command_return_trait>()?;
            input.parse::<Token![=]>()?;
            Ok(Self::CommandReturnTrait(
                key,
                input.parse::<syn::TraitBound>()?,
            ))
        } else if lookahead.peek(kw::file) {
            let key = input.parse::<kw::file>()?;
            input.parse::<Token![=]>()?;
            Ok(Self::File(key, input.parse::<syn::LitStr>()?))
        } else if lookahead.peek(kw::inline) {
            let key = input.parse::<kw::inline>()?;
            input.parse::<Token![=]>()?;
            Ok(Self::Inline(key, input.parse::<syn::LitStr>()?))
        } else {
            Err(lookahead.error())
        }
    }
}

mod kw {
    syn::custom_keyword!(serde_path);
    syn::custom_keyword!(ext_impl_ty);
    syn::custom_keyword!(command_return_ty);
    syn::custom_keyword!(command_return_trait);
    syn::custom_keyword!(file);
    syn::custom_keyword!(inline);
}
