/// Form styling.

#[derive(CSS)]
#[mercutio(css(outline = "none", border = "3px solid #555"), elements(Input))]
pub struct FormTextInputStyle;

#[derive(CSS)]
#[mercutio(css(outline = "none", border = "3px solid #555"), elements(Input))]
pub struct FormSubmitInputStyle;

#[derive(CSS)]
#[mercutio(css(padding = "5px"), elements(Form))]
pub struct FormStyle;
