pub use mercutio_codegen::CSS;

/// A trait which applies the relevant CSS styles to an item.
///
/// You can (but probably don't want to) implement this yourself, but we suggest using the derive
/// macro we provide.
///
/// ```rust
/// # use mercutio::*;
/// #[derive(CSS)]
/// #[font_family = "sans-serif"]
/// #[font_size = "24px"]
/// #[elements(H1, H2, H3)]
/// struct Title;
/// ```
pub trait Apply<T> {
    fn apply(t: T) -> T;
}
