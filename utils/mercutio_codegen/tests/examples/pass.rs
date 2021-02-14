use mercutio::*;

#[derive(CSS, Debug)]
#[mercutio(
    elements(H1, H2, H3, H4, H5, H6),
    css(font_family = "sans-serif", font_size = "24px")
)]
pub struct TitleStyles;

pub fn main() {}
