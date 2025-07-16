use heck::ToTitleCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Expr, ExprLit, Ident, ItemEnum, Lit, LitStr, Result, Variant, Visibility,
    parse::{Parse, ParseStream},
};

pub fn name(item: ItemEnum) -> TokenStream {
    let (vis, ident, variants) = (&item.vis, &item.ident, &item.variants);

    let mut match_arms = Vec::with_capacity(variants.len());
    #[cfg(feature = "name_includes_plural")]
    let mut match_arms_plural = Vec::with_capacity(variants.len());

    for variant in variants {
        let ident = &variant.ident;
        let (name, _name_plural) = find(variant, cfg!(feature = "name_includes_plural"));
        #[cfg(feature = "name_includes_plural")]
        let name_plural = _name_plural.unwrap();

        match_arms.push(quote! { Self::#ident => #name });
        #[cfg(feature = "name_includes_plural")]
        match_arms_plural.push(quote! { Self::#ident => #name_plural });
    }

    let vis = ImplVis(vis);
    let constness = ImplConstness;

    let name_method = quote! {
        #vis #constness fn name(&self) -> &'static str {
            match self {
                #(#match_arms),*
            }
        }
    };

    #[cfg(feature = "name_includes_plural")]
    let body = {
        let plural_methods = name_plural_methods(&vis, &match_arms_plural);
        quote! {
            #name_method
            #plural_methods
        }
    };

    #[cfg(not(feature = "name_includes_plural"))]
    let body = name_method;

    let impl_block = ImplBlock {
        #[cfg(feature = "name_trait")]
        trait_name: "Name",
        enum_ident: ident,
        body,
    };

    quote! { #impl_block }
}

#[cfg(not(feature = "name_includes_plural"))]
pub fn name_plural(item: ItemEnum) -> TokenStream {
    let (vis, ident, variants) = (&item.vis, &item.ident, &item.variants);

    let mut match_arms = Vec::with_capacity(variants.len());

    for variant in variants {
        let ident = &variant.ident;
        let (_, name_plural) = find(variant, true);
        let name_plural = name_plural.unwrap();

        match_arms.push(quote! { Self::#ident => #name_plural });
    }

    let vis = ImplVis(vis);
    let body = name_plural_methods(&vis, &match_arms);
    let impl_block = ImplBlock {
        #[cfg(feature = "name_trait")]
        trait_name: "NamePlural",
        enum_ident: ident,
        body,
    };

    quote! { #impl_block }
}

fn find(variant: &Variant, needs_plural: bool) -> (String, Option<String>) {
    let name = variant
        .attrs
        .iter()
        .find_map(|attr| {
            let meta = attr.meta.require_name_value().ok()?;
            if !meta.path.is_ident("name") {
                return None;
            }
            match &meta.value {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) => Some(s.value()),
                _ => None,
            }
        })
        .unwrap_or_else(|| variant.ident.to_string().to_title_case());

    if !needs_plural {
        return (name, None);
    }

    let name_plural = variant
        .attrs
        .iter()
        .find_map(|attr| {
            let meta = attr.meta.require_list().ok()?;
            if !meta.path.is_ident("name") {
                return None;
            }
            struct Inner(String);
            impl Parse for Inner {
                fn parse(input: ParseStream) -> Result<Self> {
                    syn::custom_keyword!(plural);

                    let _: plural = input.parse()?;
                    let _: syn::Token![=] = input.parse()?;
                    let s: LitStr = input.parse()?;
                    Ok(Self(s.value()))
                }
            }
            meta.parse_args::<Inner>().ok().map(|i| i.0)
        })
        .unwrap_or_else(|| {
            let mut n = name.clone();
            n.push('s');
            n
        });

    (name, Some(name_plural))
}

fn name_plural_methods(vis: &ImplVis, match_arms: &[TokenStream]) -> TokenStream {
    let constness = ImplConstness;

    quote! {
        #vis #constness fn name_plural(&self) -> &'static str {
            match self {
                #(#match_arms),*
            }
        }

        #vis #constness fn name_pluralized(&self, n: usize) -> &'static str {
            if n == 1 { self.name() } else { self.name_plural() }
        }
    }
}

struct ImplBlock<'a, T> {
    #[cfg(feature = "name_trait")]
    trait_name: &'static str,
    enum_ident: &'a Ident,
    body: T,
}

impl<T: ToTokens> ToTokens for ImplBlock<'_, T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            #[cfg(feature = "name_trait")]
            trait_name,
            enum_ident,
            body,
        } = self;

        #[cfg(feature = "name_trait")]
        {
            let trait_ident = Ident::new(trait_name, proc_macro2::Span::call_site());
            quote! { impl #trait_ident for #enum_ident { #body } }.to_tokens(tokens)
        }
        #[cfg(not(feature = "name_trait"))]
        {
            quote! { impl #enum_ident { #body } }.to_tokens(tokens);
        }
    }
}

struct ImplVis<'a>(&'a Visibility);

impl ToTokens for ImplVis<'_> {
    fn to_tokens(&self, _tokens: &mut TokenStream) {
        let _vis = self.0;
        #[cfg(not(feature = "name_trait"))]
        quote! { #_vis }.to_tokens(_tokens)
    }
}

struct ImplConstness;

impl ToTokens for ImplConstness {
    fn to_tokens(&self, _tokens: &mut TokenStream) {
        #[cfg(not(feature = "name_trait"))]
        quote! { const }.to_tokens(_tokens)
    }
}
