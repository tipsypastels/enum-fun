use heck::ToTitleCase;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::{borrow::Cow, collections::HashMap, iter};
use syn::{
    Attribute, Error, Ident, ItemEnum, LitStr, Result, Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
};

pub fn name(item: ItemEnum) -> Result<TokenStream> {
    let (vis, attrs, ident, variants) = (&item.vis, &item.attrs, &item.ident, &item.variants);
    let formats = Formats::new(item.span(), attrs)?;
    let mut arms = HashMap::<&str, Vec<TokenStream>>::new();

    for variant in variants {
        let variant_ident = &variant.ident;
        let overrides = Overrides::new(&formats, &variant.attrs)?;

        for (key, format) in formats.iter() {
            let value = overrides
                .get(key)
                .map(Cow::Borrowed)
                .or_else(|| {
                    (key != "base")
                        .then(|| {
                            overrides
                                .get("base")
                                .map(|s| Cow::Owned(format.transform(s)))
                        })
                        .flatten()
                })
                .unwrap_or_else(|| Cow::Owned(format.transform(&variant_ident.to_string())));

            arms.entry(key)
                .and_modify(|v| v.push(quote! { Self::#variant_ident => #value }))
                .or_insert_with(|| vec![quote! { Self::#variant_ident => #value }]);
        }
    }

    let mut body = Vec::new();

    for (&key, variant_arms) in &arms {
        let method_name = method_name_for_key(key);
        let method_ident = Ident::new(&method_name, Span::call_site());

        body.push(quote! {
            #vis const fn #method_ident(&self) -> &'static str {
                match self {
                    #(#variant_arms),*
                }
            }
        });
    }

    for (singular, plural) in &formats.pluralizers {
        let singular_method_name = method_name_for_key(singular);
        let plural_method_name = method_name_for_key(plural);
        let method_name = pluralizer_method_name_for_key(singular);

        let singular_method_ident = Ident::new(&singular_method_name, Span::call_site());
        let plural_method_ident = Ident::new(&plural_method_name, Span::call_site());
        let method_ident = Ident::new(&method_name, Span::call_site());

        body.push(quote! {
            #vis const fn #method_ident(&self, n: usize) -> &'static str {
                if n == 1 {
                    self.#singular_method_ident()
                } else {
                    self.#plural_method_ident()
                }
            }
        })
    }

    Ok(quote! {
        impl #ident {
            #(#body)*
        }
    })
}

struct Formats {
    base: Format,
    extras: HashMap<String, Format>,
    pluralizers: Vec<(String, String)>,
}

impl Formats {
    fn new(span: Span, attrs: &[Attribute]) -> Result<Self> {
        let mut base = None;
        let mut extras = HashMap::new();
        let mut pluralizers = Vec::new();

        for attr in attrs {
            if !attr.path().is_ident("name") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("base") {
                    let value = meta.value()?;
                    let format = value.parse()?;

                    base = Some(format);
                    return Ok(());
                }

                if meta.path.is_ident("extra") {
                    return meta.parse_nested_meta(|meta| {
                        if meta.path.is_ident("base") {
                            return Err(Error::new(meta.path.span(), "invalid extra key `base`"));
                        }

                        let key = meta
                            .path
                            .get_ident()
                            .ok_or_else(|| Error::new(meta.path.span(), "expected ident"))?
                            .to_string();

                        let value = meta.value()?;
                        let format = value.parse()?;

                        extras.insert(key, format);
                        Ok(())
                    });
                }

                if meta.path.is_ident("pluralizer") {
                    struct IdentPair(Ident, Ident);
                    impl Parse for IdentPair {
                        fn parse(input: ParseStream) -> Result<Self> {
                            let a = input.parse()?;
                            let _: Token![,] = input.parse()?;
                            let b = input.parse()?;
                            Ok(Self(a, b))
                        }
                    }

                    let content;
                    parenthesized!(content in meta.input);
                    let idents: IdentPair = content.parse()?;
                    let keys = (idents.0, idents.1);

                    pluralizers.push(keys);
                    return Ok(());
                }

                Err(meta.error("unknown attribute"))
            })?;
        }

        let base = base.ok_or_else(|| Error::new(span, "missing `#[name(base = \"...\")]`"))?;
        let pluralizers = pluralizers
            .into_iter()
            .map(|(singular, plural)| {
                let singular_s = singular.to_string();
                let plural_s = plural.to_string();

                for (ident, str) in [(singular, &singular_s), (plural, &plural_s)] {
                    if str == "base" {
                        continue;
                    }
                    if !extras.contains_key(str) {
                        return Err(Error::new(ident.span(), format!("unknown key `{str}`")));
                    }
                }
                Ok((singular_s, plural_s))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            base,
            extras,
            pluralizers,
        })
    }

    fn has(&self, key: &str) -> bool {
        key == "base" || self.extras.contains_key(key)
    }

    fn get(&self, key: &str) -> Format {
        if key == "base" {
            self.base
        } else {
            self.extras.get(key).copied().unwrap()
        }
    }

    fn iter(&self) -> impl Iterator<Item = (&str, Format)> {
        iter::once(("base", self.base)).chain(self.extras.iter().map(|(k, v)| (k.as_str(), *v)))
    }
}

#[derive(Copy, Clone)]
enum Format {
    TitleCase { lower: bool, plural: bool },
}

impl Format {
    const VALID_INPUTS: [&str; 4] = [
        "title case",
        "title case lower",
        "title case plural",
        "title case lower plural",
    ];

    fn parse_str(span: Span, s: &str) -> Result<Self> {
        match s {
            "title case" => Ok(Self::TitleCase {
                lower: false,
                plural: false,
            }),
            "title case lower" => Ok(Self::TitleCase {
                lower: true,
                plural: false,
            }),
            "title case plural" => Ok(Self::TitleCase {
                lower: false,
                plural: true,
            }),
            "title case lower plural" => Ok(Self::TitleCase {
                lower: true,
                plural: true,
            }),
            _ => Err(Error::new(
                span,
                format!(
                    "expected one of {}",
                    Self::VALID_INPUTS
                        .into_iter()
                        .map(|s| format!("\"{s}\""))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )),
        }
    }

    fn transform(&self, s: &str) -> String {
        match self {
            Self::TitleCase {
                lower: false,
                plural: false,
            } => s.to_title_case(),
            Self::TitleCase {
                lower: true,
                plural: false,
            } => s.to_title_case().to_lowercase(),
            Self::TitleCase {
                lower: false,
                plural: true,
            } => {
                format!("{}s", s.to_title_case())
            }
            Self::TitleCase {
                lower: true,
                plural: true,
            } => {
                format!("{}s", s.to_title_case().to_lowercase())
            }
        }
    }
}

impl Parse for Format {
    fn parse(input: ParseStream) -> Result<Self> {
        let s: LitStr = input.parse()?;
        Self::parse_str(s.span(), &s.value())
    }
}

struct Overrides {
    map: HashMap<String, String>,
}

impl Overrides {
    fn new(formats: &Formats, attrs: &[Attribute]) -> Result<Self> {
        let mut map = HashMap::new();

        for attr in attrs {
            if !attr.path().is_ident("name") {
                continue;
            }

            struct Override(Ident, String);

            impl Parse for Override {
                fn parse(input: ParseStream) -> Result<Self> {
                    let ident: Ident = input.parse()?;
                    let _: Token![=] = input.parse()?;
                    let value: LitStr = input.parse()?;

                    Ok(Self(ident, value.value()))
                }
            }

            let parser = Punctuated::<Override, Token![,]>::parse_separated_nonempty;
            let entries = attr.parse_args_with(parser)?;

            for entry in entries {
                let Override(ident, value) = entry;
                let key = ident.to_string();

                if !formats.has(&key) {
                    return Err(Error::new(ident.span(), format!("unknown key `{key}`")));
                }

                map.insert(key, value);
            }
        }

        Ok(Self { map })
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(|s| s.as_str())
    }
}

fn method_name_for_key(key: &str) -> Cow<'static, str> {
    if key == "base" {
        Cow::Borrowed("name")
    } else {
        Cow::Owned(format!("name_{key}"))
    }
}

fn pluralizer_method_name_for_key(key: &str) -> Cow<'static, str> {
    if key == "base" {
        Cow::Borrowed("name_pluralized")
    } else {
        Cow::Owned(format!("name_{key}_pluralized"))
    }
}
