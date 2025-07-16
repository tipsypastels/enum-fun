//! Silly enum helpers.

#[cfg(feature = "name-trait")]
pub trait Name {
    fn name(&self) -> &'static str;

    #[cfg(feature = "name-includes-plural")]
    fn name_plural(&self) -> &'static str;

    #[cfg(feature = "name-includes-plural")]
    fn name_pluralized(&self, count: usize) -> &'static str;
}

#[cfg(all(feature = "name-trait", not(feature = "name-includes-plural")))]
pub trait NamePlural: Name {
    fn name_plural(&self) -> &'static str;
    fn name_pluralized(&self, count: usize) -> &'static str;
}

/// Generate a method `name(&self) -> &'static str` that
/// returns the name of a given enum variant.
///
/// If the `name-trait` feature is enabled, an implementation
/// of a `Name` trait is generated. Otherwise, an inherent
/// implementation is used.
///
/// The name of an enum variant is its identifier in Title Case.
/// `Foo` becomes `"Foo"` and `HelloWorld` becomes `"Hello World"`.
/// This can be overridden by using the `#[name = "..."]` attribute
/// on a variant, e.g.:
///
/// ```rust
/// use enum_fun::Name;
///
/// #[derive(Name)]
/// enum Words {
///     Foo,
///     HelloWorld,
///     #[name = "Baz"]
///     Bar,
/// }
///
/// use Words::*;
///
/// assert_eq!(Foo.name(), "Foo");
/// assert_eq!(HelloWorld.name(), "Hello World");
/// assert_eq!(Bar.name(), "Baz");
/// ```
pub use enum_fun_macros::Name;

/// Generate a method `name_plural(&self) -> &'static str` that
/// returns the pluralized name of a given enum variant.
///
/// If the `name-trait` feature is enabled, an implementation
/// of a `NamePlural` trait is generated. Otherwise,
/// an inherent implementation is used.
///
/// The plural name of an enum variant is the same name that
/// [`derive(Name)`](derive@Name) gave it, with an additional
/// `"s"` appended. `Foo` becomes `"Foos"` and `HelloWorld`
/// becomes `"Hello Worlds"`. This can be overridden by using
/// the `#[name(plural = "...")]` attribute on a variant, e.g.:
///
/// ```rust
/// use enum_fun::{Name, NamePlural};
///
/// #[derive(Name, NamePlural)]
/// enum Words {
///     Foo,
///     HelloWorld,
///     #[name = "Baz"]
///     Bar,
///     #[name(plural = "Quuxes")]
///     Quux,
/// }
///
/// use Words::*;
///
/// assert_eq!(Foo.name_plural(), "Foos");
/// assert_eq!(HelloWorld.name_plural(), "Hello Worlds");
/// assert_eq!(Bar.name_plural(), "Bazs");
/// assert_eq!(Quux.name_plural(), "Quuxes");
/// ```
///
/// A utility method `name_pluralized(&self, n: usize) -> &'static str`
/// is also generated. It will return `name()` if `n`
/// is `1`, and `name_plural()` otherwise. It does not
/// prepend the provided number to the string.
///
/// ```rust
/// # use enum_fun::{Name, NamePlural};
/// # #[derive(Name, NamePlural)]
/// # enum Words {
/// #     Foo,
/// # }
/// # use Words::Foo;
/// assert_eq!(Foo.name_pluralized(1), "Foo");
/// assert_eq!(Foo.name_pluralized(/* anything non-1 */ 2), "Foos");
/// ```
///
/// If the `name-includes-plural` feature is enabled,
/// `NamePlural` does not exist and all of its features
/// are part of [`Name`](derive@Name) instead. You may use
/// this if all your named enums must also support
/// pluralization, to save a derive.
#[cfg(not(feature = "name-includes-plural"))]
pub use enum_fun_macros::NamePlural;

/// Generates inherent constants and an iterator method
/// to enable iterating and indexing the enum variants.
///
/// ```rust
/// use enum_fun::Variants;
///
/// #[derive(Debug, PartialEq, Variants)]
/// enum Words {
///     Foo,
///     Bar,
///     Baz,
/// }
///
/// use Words::*;
///
/// assert_eq!(Words::VARIANT_COUNT, 3);
/// assert_eq!(Words::VARIANTS, [Foo, Bar, Baz]);
/// assert_eq!(Words::variants().collect::<Vec<_>>(), vec![Foo, Bar, Baz]);
/// ```
///
/// The return type of the `variants()` method is an `impl ExactSizeIterator<Item = Self>`.
///
/// Only enums without fields are supported.
pub use enum_fun_macros::Variants;
