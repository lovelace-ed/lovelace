/// Colous. Note that these may or may not be suitable for your application.

#[derive(CSS, Debug)]
#[elements(Div)]
#[background_color = "#f4d03f"]
/// Applies a yellow background to a `Div`.
pub struct YellowBackground;

#[derive(CSS, Debug)]
#[elements(Div)]
#[background_color = "#d5dbdb"]
/// Applies a grey background to a `Div`.
pub struct GreyBackground;
