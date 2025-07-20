mod name;
mod predicates;
mod variants;

use proc_macro::TokenStream;
use syn::{ItemEnum, parse_macro_input};

#[proc_macro_derive(Name, attributes(name))]
pub fn name(input: TokenStream) -> TokenStream {
    name::name(parse_macro_input!(input as ItemEnum))
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

#[proc_macro_derive(Predicates)]
pub fn predicates(input: TokenStream) -> TokenStream {
    predicates::predicates(parse_macro_input!(input as ItemEnum)).into()
}

#[proc_macro_derive(Variants)]
pub fn variants(input: TokenStream) -> TokenStream {
    variants::variants(parse_macro_input!(input as ItemEnum))
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
