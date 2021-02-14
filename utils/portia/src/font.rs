/// Font styles.

#[derive(CSS, Debug)]
#[mercutio(
    elements(H1, H2, H3, H4, H5, H6),
    css(font_family = "sans-serif", font_size = "24px")
)]
/// A small title.
pub struct SmallTitle;
