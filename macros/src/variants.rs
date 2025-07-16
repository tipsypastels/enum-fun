use itertools::{EitherOrBoth::*, Itertools};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Error, Fields, Ident, ItemEnum, Result, Variant, punctuated::Punctuated, spanned::Spanned,
    token::Comma,
};

type Variants = Punctuated<Variant, Comma>;

pub fn variants(item: ItemEnum) -> Result<TokenStream> {
    let (item_vis, item_ident, item_variants) = (&item.vis, &item.ident, &item.variants);
    let iter_ident = Ident::new(&format!("{item_ident}Variants"), item_ident.span());

    let first_ident = first_ident(item_variants, item.span())?;
    let match_arms = zip_idents(item_variants)
        .map(|item| match item? {
            (curr, Some(next)) => Ok(quote!(#item_ident::#curr => Some(#item_ident::#next))),
            (curr, None) => Ok(quote!(#item_ident::#curr => None)),
        })
        .collect::<Result<Vec<_>>>()?;

    let variant_count = match_arms.len();

    Ok(quote! {
        impl #item_ident {
            #item_vis const VARIANT_COUNT: usize = #variant_count;

            #item_vis const fn variants() -> #iter_ident {
                #iter_ident(Some(#item_ident::#first_ident))
            }
        }

        #item_vis struct #iter_ident(Option<#item_ident>);

        impl Iterator for #iter_ident {
            type Item = #item_ident;

            fn next(&mut self) -> Option<Self::Item> {
                let curr = self.0.as_ref()?;
                let next = match curr { #(#match_arms),* };

                if let Some(next) = next {
                    self.0.replace(next)
                } else {
                    self.0.take()
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                (#variant_count, Some(#variant_count))
            }
        }

        impl ExactSizeIterator for #iter_ident {}
    })
}

fn first_ident(variants: &Variants, span: Span) -> Result<&Ident> {
    unit_ident(
        variants
            .iter()
            .next()
            .ok_or(Error::new(span, "must not be empty"))?,
    )
}

fn unit_ident(variant: &Variant) -> Result<&Ident> {
    if let Fields::Unit = variant.fields {
        Ok(&variant.ident)
    } else {
        Err(Error::new(variant.span(), "must be unit"))
    }
}

fn zip_idents(variants: &Variants) -> impl Iterator<Item = Result<(&Ident, Option<&Ident>)>> {
    variants
        .iter()
        .zip_longest(variants.iter().skip(1))
        .map(|item| match item {
            Both(curr, next) => Ok((unit_ident(curr)?, Some(unit_ident(next)?))),
            Left(curr) => Ok((unit_ident(curr)?, None)),
            Right(_) => unreachable!(),
        })
}
