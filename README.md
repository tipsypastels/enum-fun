# enum-fun

[![Crates.io](https://img.shields.io/crates/v/enum-fun)](https://crates.io/crates/enum-fun)
[![docs](https://docs.rs/enum-fun/badge.svg)](https://docs.rs/enum-fun)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/tipsypastels/enum-fun/blob/main/LICENSE)

Silly enum helpers.

## `derive(Name)`

For each variant of an enum, generates an inherent `const fn name(&self) -> &'static str`. If the `name-trait` feature is enabled, an implementation of a `Name` trait is generated instead of an inherent function.

The name of an enum variant is its identifier in Title Case. `Foo` becomes `"Foo"` and `HelloWorld` becomes `"Hello World"`. This can be overridden by using the `#[name = "..."]` attribute on a variant, e.g.:

```rust
# use enum_fun::Name;
#[derive(Name)]
enum Words {
  Foo,
  #[name = "Baz"]
  Bar,
}
```

## `derive(NamePlural)`

An extension to `Name`. Generates inherent functions `const fn name_plural(&self) -> &'static str` and `const fn name_pluralized(&self, n: usize) -> &'static str`. If the `name-trait` feature is enabled, an implementation of a `NamePlural: Name` trait is generated instead of inherent functions.

The plural name of an enum is the same name that `derive(Name)` gave it, with an additional `"s"` appended. `Foo` becomes `"Foos"` and `HelloWorld` becomes `"Hello Worlds"`. This can be overriden by using the `#[name(plural = "...")]` attribute on a variant, e.g.:

```rust
# use enum_fun::{Name, NamePlural};
#[derive(Name, NamePlural)]
enum Words {
  Foo,
  Bar,
  #[name(plural = "Bazes")]
  Baz,
}
```

The generated function `name_pluralized(&self, n: usize) -> &'static str` will return `name()` if `n == 1` and `name_plural()` otherwise. It does not prepend the provided number to the string.

If the `name-includes-plural` feature is enabled, `NamePlural` does not exist and all of its features are a part of `Name` instead. You may use this if all of your named enums must also support pluralization, to save a derive.

## `derive(Variants)`

Generates an inherent `const VARIANTS_COUNT: usize`, `const VARIANTS: [Self; Self::VARIANTS_COUNT]` and `fn variants() -> impl ExactSizeIterator<Item = Self>` for an enum. This allows iterating and indexing the enum members.

Only enums without fields are supported.