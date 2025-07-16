use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Fields, ItemEnum, Result, spanned::Spanned};

pub fn variants(item: ItemEnum) -> Result<TokenStream> {
    let (vis, ident, variants) = (&item.vis, &item.ident, &item.variants);
    let variant_idents = variants
        .iter()
        .map(|variant| {
            let &Fields::Unit = &variant.fields else {
                return Err(Error::new(
                    variant.fields.span(),
                    "expected enum without fields",
                ));
            };
            Ok(&variant.ident)
        })
        .collect::<Result<Vec<_>>>()?;

    let variant_count = variant_idents.len();

    Ok(quote! {
        impl #ident {
            #vis const VARIANT_COUNT: usize = #variant_count;
            #vis const VARIANTS: [Self; #variant_count] = [#(Self::#variant_idents),*];

            #vis fn variants() -> impl std::iter::ExactSizeIterator<Item = Self> {
                [#(Self::#variant_idents),*].into_iter()
            }
        }
    })
}
