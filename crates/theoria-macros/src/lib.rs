//! Proc-macros for theoria.
//!
//! `#[story]` turns a function whose typed parameters are the editable "args"
//! into a `theoria::Story` carrying those args (as live controls), the captured
//! body source (for "show code"), and the doc-comment description (for autodocs).
//! It is theoria's stand-in for Storybook's CSF + react-docgen, which Rust can't
//! provide via runtime reflection.
//!
//! ```ignore
//! #[story(name = "Canvas/Yard", yard_w = 70.0, yard_d = 30.0)]
//! /// The yard canvas, drawn to scale.
//! fn yard(yard_w: f64, yard_d: f64) -> impl IntoView {
//!     view! { <Yard yard_w=yard_w yard_d=yard_d px_ft=12.0 pad=40.0 /> }
//! }
//! ```
//!
//! Supported arg types: `bool` (toggle), `f64` (number), `String` (text).

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{
    Expr, FnArg, ItemFn, Lit, Meta, Pat, Token, Type, parse_macro_input, punctuated::Punctuated,
    spanned::Spanned,
};

enum Kind {
    Bool,
    Num,
    Text,
}

#[proc_macro_attribute]
pub fn story(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let overrides = parse_macro_input!(attr with Punctuated::<Meta, Token![,]>::parse_terminated);

    // `name = "..."` sets the story name; any other `key = expr` is a default for
    // the matching arg.
    let mut name_override: Option<Expr> = None;
    let mut defaults: HashMap<String, Expr> = HashMap::new();
    for meta in overrides {
        if let Meta::NameValue(nv) = meta {
            let key = nv
                .path
                .get_ident()
                .map(ToString::to_string)
                .unwrap_or_default();
            if key == "name" {
                name_override = Some(nv.value);
            } else {
                defaults.insert(key, nv.value);
            }
        }
    }

    let fname = func.sig.ident.clone();
    let story_name: Expr = name_override.unwrap_or_else(|| {
        let s = fname.to_string();
        syn::parse_quote!(#s)
    });
    let description = doc_description(&func);

    let mut sig_lets = Vec::new();
    let mut get_lets = Vec::new();
    let mut arg_entries = Vec::new();
    for input in &func.sig.inputs {
        let FnArg::Typed(pt) = input else {
            return err(input, "#[story] functions take no self");
        };
        let Pat::Ident(pi) = &*pt.pat else {
            return err(&pt.pat, "#[story] args must be plain identifiers");
        };
        let ident = pi.ident.clone();
        let name = ident.to_string();
        let sig_ident = format_ident!("__sig_{}", ident);
        let (variant, default): (proc_macro2::TokenStream, Expr) = match type_kind(&pt.ty) {
            Some(Kind::Bool) => (
                quote!(Bool),
                defaults
                    .get(&name)
                    .cloned()
                    .unwrap_or_else(|| syn::parse_quote!(false)),
            ),
            Some(Kind::Num) => (
                quote!(Num),
                defaults
                    .get(&name)
                    .cloned()
                    .unwrap_or_else(|| syn::parse_quote!(0.0)),
            ),
            Some(Kind::Text) => {
                let d = defaults
                    .get(&name)
                    .cloned()
                    .unwrap_or_else(|| syn::parse_quote!(""));
                (
                    quote!(Text),
                    syn::parse_quote!(::std::string::String::from(#d)),
                )
            }
            None => return err(&pt.ty, "#[story] arg type must be bool, f64, or String"),
        };
        sig_lets.push(quote!(let #sig_ident = leptos::prelude::RwSignal::new(#default);));
        get_lets.push(quote!(let #ident = #sig_ident.get();));
        arg_entries.push(quote!((#name, theoria::ArgControl::#variant(#sig_ident))));
    }

    let body = &func.block;
    let source = body_source(&func);

    quote! {
        pub fn #fname() -> theoria::Story {
            #[allow(unused_imports)]
            use leptos::prelude::*;
            #(#sig_lets)*
            let __view = move || {
                #(#get_lets)*
                leptos::prelude::IntoAny::into_any(#body)
            };
            theoria::Story::__from_macro(
                #story_name,
                __view,
                ::std::vec![#(#arg_entries),*],
                #source,
                #description,
            )
        }
    }
    .into()
}

fn type_kind(ty: &Type) -> Option<Kind> {
    if let Type::Path(tp) = ty {
        match tp.path.segments.last()?.ident.to_string().as_str() {
            "bool" => return Some(Kind::Bool),
            "f64" => return Some(Kind::Num),
            "String" => return Some(Kind::Text),
            _ => {}
        }
    }
    None
}

/// The doc-comment lines joined into a `Some("...")` / `None` expression.
fn doc_description(func: &ItemFn) -> Expr {
    let mut doc = String::new();
    for a in &func.attrs {
        if a.path().is_ident("doc")
            && let Meta::NameValue(nv) = &a.meta
            && let Expr::Lit(syn::ExprLit {
                lit: Lit::Str(s), ..
            }) = &nv.value
        {
            if !doc.is_empty() {
                doc.push('\n');
            }
            doc.push_str(s.value().trim());
        }
    }
    if doc.is_empty() {
        syn::parse_quote!(::std::option::Option::None)
    } else {
        syn::parse_quote!(::std::option::Option::Some(#doc))
    }
}

/// The body's source text (braces stripped) as a string literal, for "show
/// code". Falls back to the token stream when span source text is unavailable.
fn body_source(func: &ItemFn) -> Expr {
    let raw = func
        .block
        .span()
        .source_text()
        .unwrap_or_else(|| quote!(#func).to_string());
    let s = raw
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim()
        .to_string();
    syn::parse_quote!(#s)
}

fn err(tokens: impl quote::ToTokens, msg: &str) -> TokenStream {
    syn::Error::new(tokens.span(), msg)
        .to_compile_error()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_kind_maps_supported_types() {
        assert!(matches!(
            type_kind(&syn::parse_quote!(bool)),
            Some(Kind::Bool)
        ));
        assert!(matches!(
            type_kind(&syn::parse_quote!(f64)),
            Some(Kind::Num)
        ));
        assert!(matches!(
            type_kind(&syn::parse_quote!(String)),
            Some(Kind::Text)
        ));
        // Path-qualified forms resolve by the final segment.
        assert!(matches!(
            type_kind(&syn::parse_quote!(std::string::String)),
            Some(Kind::Text)
        ));
    }

    #[test]
    fn type_kind_rejects_unsupported_types() {
        assert!(type_kind(&syn::parse_quote!(i32)).is_none());
        assert!(type_kind(&syn::parse_quote!(Signal<bool>)).is_none());
        assert!(type_kind(&syn::parse_quote!(&str)).is_none());
    }
}
