/// Colous. Note that these may or may not be suitable for your application.

#[derive(CSS, Debug)]
#[mercutio(
    css(background_color = "#f4d03f"),
    elements(Div)
)]
/// Applies a yellow background to a `Div`.
pub struct YellowBackground;

#[derive(CSS, Debug)]
#[mercutio(
    css(background_color = "#d5dbdb"),
    elements(Div)
)]
/// Applies a grey background to a `Div`.
pub struct GreyBackground;
