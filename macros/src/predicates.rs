use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, ItemEnum};

pub fn predicates(item: ItemEnum) -> TokenStream {
    let (vis, ident, variants) = (&item.vis, &item.ident, &item.variants);
    let methods = variants.iter().map(|variant| {
        let ident = &variant.ident;
        let predicate_name = format!("is_{}", ident.to_string().to_snake_case());
        let predicate = Ident::new(&predicate_name, ident.span());

        quote! {
            #vis const fn #predicate(&self) -> bool {
                matches!(self, Self::#ident { .. })
            }
        }
    });
    quote! {
        impl #ident {
            #(#methods)*
        }
    }
}
