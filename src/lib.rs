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

pub use enum_fun_macros::{Name, Variants};

#[cfg(not(feature = "name-includes-plural"))]
pub use enum_fun_macros::NamePlural;
