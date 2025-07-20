#![warn(missing_docs)]

//! Silly enum helpers.

pub use enum_fun_macros::Name;

/// Generates `is_*` predicate methods for each enum variant.
///
/// The name of a predicate method is `is_` followed by the variant's
/// identifier in snake_case. `Foo` becomes `is_foo` and `HelloWorld`
/// becomes `is_hello_world`.
///
/// ```rust
/// use enum_fun::Predicates;
///
/// #[derive(Predicates)]
/// enum Words {
///     Foo,
///     Bar,
/// #   SnakeCaseTest,
/// }
///
/// use Words::*;
///
/// assert!(Foo.is_foo());
/// assert!(!Foo.is_bar());
///
/// assert!(Bar.is_bar());
/// assert!(!Bar.is_foo());
/// #
/// # assert!(SnakeCaseTest.is_snake_case_test());
/// ```
pub use enum_fun_macros::Predicates;

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
