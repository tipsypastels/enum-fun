mod name;
mod variants;

use proc_macro::TokenStream;
use syn::{ItemEnum, parse_macro_input};

#[proc_macro_derive(Name, attributes(name))]
pub fn name(input: TokenStream) -> TokenStream {
    name::name(parse_macro_input!(input as ItemEnum)).into()
}

#[cfg(not(feature = "name-includes-plural"))]
#[proc_macro_derive(NamePlural)]
pub fn name_plural(input: TokenStream) -> TokenStream {
    name::name_plural(parse_macro_input!(input as ItemEnum)).into()
}

#[proc_macro_derive(Variants)]
pub fn variants(input: TokenStream) -> TokenStream {
    variants::variants(parse_macro_input!(input as ItemEnum))
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
