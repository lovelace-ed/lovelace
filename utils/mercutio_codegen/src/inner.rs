use darling::FromDeriveInput;
use syn::DeriveInput;

pub fn css_inner(input: DeriveInput) -> proc_macro2::TokenStream {
    let css_props: CssProps = match CssProps::from_derive_input(&input) {
        Ok(ok) => ok,
        Err(e) => {
            return e.write_errors();
        }
    };
    css_props
        .css
        .to_tokens(input.ident, css_props.elements, css_props.use_classes)
}

#[derive(FromDeriveInput)]
#[darling(attributes(mercutio))]
pub struct CssProps {
    css: CssPropsInner,
    elements: Elements,
    #[darling(default)]
    use_classes: bool,
}

impl Default for CssProps {
    fn default() -> Self {
        Self {
            css: Default::default(),
            elements: Default::default(),
            use_classes: false,
        }
    }
}

#[derive(FromMeta, Default)]
#[darling(default)]
pub struct Elements {
    #[darling(rename = "H1")]
    h1: bool,
    #[darling(rename = "H2")]
    h2: bool,
    #[darling(rename = "H3")]
    h3: bool,
    #[darling(rename = "H4")]
    h4: bool,
    #[darling(rename = "H5")]
    h5: bool,
    #[darling(rename = "H6")]
    h6: bool,
    #[darling(rename = "Div")]
    div: bool,
    #[darling(rename = "Form")]
    form: bool,
    #[darling(rename = "Input")]
    input: bool,
    #[darling(rename = "Body")]
    body: bool,
}

impl From<Elements> for Vec<String> {
    fn from(elements: Elements) -> Self {
        let mut res = vec![];
        if elements.h1 {
            res.push("H1");
        }
        if elements.h2 {
            res.push("H2");
        }
        if elements.h3 {
            res.push("H3");
        }
        if elements.h4 {
            res.push("H4");
        }
        if elements.h5 {
            res.push("H5");
        }
        if elements.h6 {
            res.push("H6");
        }
        if elements.div {
            res.push("Div");
        }
        if elements.form {
            res.push("Form");
        }
        if elements.input {
            res.push("Input");
        }
        if elements.body {
            res.push("Body");
        }
        res.into_iter().map(From::from).collect()
    }
}

#[derive(Default, FromMeta)]
#[darling(default)]
pub struct CssPropsInner {
    binding: Option<String>,
    width: Option<String>,
    list_style_image: Option<String>,
    position: Option<String>,
    overflow: Option<String>,
    unicode_bidi: Option<String>,
    marker_mid: Option<String>,
    line_height: Option<String>,
    cue_before: Option<String>,
    caption_side: Option<String>,
    border_right_color: Option<String>,
    orphans: Option<String>,
    list_style_type: Option<String>,
    font_size_adjust: Option<String>,
    page_break_inside: Option<String>,
    border_top_width: Option<String>,
    border_top_style: Option<String>,
    max_width: Option<String>,
    voice_family: Option<String>,
    border_style: Option<String>,
    speak: Option<String>,
    azimuth: Option<String>,
    color: Option<String>,
    direction: Option<String>,
    flex_direction: Option<String>,
    alignment_baseline: Option<String>,
    border_left: Option<String>,
    text_align: Option<String>,
    border_bottom_width: Option<String>,
    border_bottom: Option<String>,
    min_width: Option<String>,
    margin_left: Option<String>,
    pause_before: Option<String>,
    speak_numeral: Option<String>,
    pitch_range: Option<String>,
    property_name: Option<String>,
    font_stretch: Option<String>,
    padding_top: Option<String>,
    margin: Option<String>,
    empty_cells: Option<String>,
    cursor: Option<String>,
    border_top: Option<String>,
    dominant_baseline: Option<String>,
    border_right: Option<String>,
    min_height: Option<String>,
    padding_bottom: Option<String>,
    border_collapse: Option<String>,
    bottom: Option<String>,
    background_color: Option<String>,
    text_shadow: Option<String>,
    content: Option<String>,
    border_spacing: Option<String>,
    quotes: Option<String>,
    speak_header: Option<String>,
    counter_reset: Option<String>,
    clear: Option<String>,
    table_layout: Option<String>,
    border_left_style: Option<String>,
    baseline_shift: Option<String>,
    height: Option<String>,
    word_spacing: Option<String>,
    border_left_width: Option<String>,
    z_index: Option<String>,
    marker_end: Option<String>,
    letter_spacing: Option<String>,
    border_left_color: Option<String>,
    display: Option<String>,
    border_right_style: Option<String>,
    font: Option<String>,
    page_break_before: Option<String>,
    text_transform: Option<String>,
    richness: Option<String>,
    background_repeat: Option<String>,
    float: Option<String>,
    white_space: Option<String>,
    border_bottom_color: Option<String>,
    play_during: Option<String>,
    cue: Option<String>,
    outline_style: Option<String>,
    outline: Option<String>,
    border_top_color: Option<String>,
    font_size: Option<String>,
    name: Option<String>,
    counter_increment: Option<String>,
    volume: Option<String>,
    vertical_align: Option<String>,
    padding: Option<String>,
    speak_punctuation: Option<String>,
    glyph_orientation_horizontal: Option<String>,
    marker_offset: Option<String>,
    margin_top: Option<String>,
    left: Option<String>,
    page_break_after: Option<String>,
    border_bottom_style: Option<String>,
    margin_bottom: Option<String>,
    text_indent: Option<String>,
    border_width: Option<String>,
    pause_after: Option<String>,
    rendering_intent: Option<String>,
    margin_right: Option<String>,
    clip: Option<String>,
    pitch: Option<String>,
    list_style_position: Option<String>,
    border: Option<String>,
    background: Option<String>,
    speech_rate: Option<String>,
    kerning: Option<String>,
    font_style: Option<String>,
    top: Option<String>,
    cue_after: Option<String>,
    background_attachment: Option<String>,
    right: Option<String>,
    max_height: Option<String>,
    font_family: Option<String>,
    background_image: Option<String>,
    background_position: Option<String>,
    border_right_width: Option<String>,
    text_anchor: Option<String>,
    writing_mode: Option<String>,
    marker_start: Option<String>,
    glyph_orientation_vertical: Option<String>,
    padding_right: Option<String>,
    src: Option<String>,
    visibility: Option<String>,
    outline_color: Option<String>,
    font_variant: Option<String>,
    text_decoration: Option<String>,
    stress: Option<String>,
    widows: Option<String>,
    outline_width: Option<String>,
    border_color: Option<String>,
    font_weight: Option<String>,
    pause: Option<String>,
    marker: Option<String>,
    padding_left: Option<String>,
    list_style: Option<String>,
    elevation: Option<String>,
}

impl CssPropsInner {
    fn to_tokens(
        &self,
        name: syn::Ident,
        elements: Elements,
        use_classes: bool,
    ) -> proc_macro2::TokenStream {
        if use_classes {
            let alphabet: [char; 16] = [
                '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
            ];
            let class_name = nanoid!(10, &alphabet);
            let head_tokens = CssPropsInnerClassOutputter(&self, class_name.clone()).to_string();
            let x: Vec<String> = From::from(elements);
            x.into_iter()
                .map(|segment: String| {
                    let segment = format_ident!("{}", segment);
                    quote! {
                        impl ::mercutio::Apply<#name> for ::malvolio::prelude::#segment {
                            fn apply(self, _: #name) -> malvolio::prelude::#segment {
                                let string: std::borrow::Cow<'static, str> =
                                    if let Some(x) = self.read_attribute("class") {
                                        format!("{} {}", #class_name, x).into()
                                    } else {
                                        ::malvolio::prelude::Class::from("class_name")
                                    };
                                self.attribute(::malvolio::prelude::Class::from(string))
                            }
                        }
                    }
                })
                .fold(
                    quote! {
                        impl Apply<#name> for ::malvolio::prelude::Head {
                            fn apply(self, _: #name) -> ::malvolio::prelude::Head {
                                self.child(StyleTag::new(#head_tokens))
                            }
                        }
                    },
                    |a, b| quote! {#a #b},
                )
        } else {
            let tokens = CssPropsInnerStyleOutputter(&self).to_string();
            let x: Vec<String> = From::from(elements);
            x.into_iter()
                .map(|segment: String| {
                    let segment = format_ident!("{}", segment);
                    quote! {
                        impl ::mercutio::Apply<#name> for ::malvolio::prelude::#segment {
                            fn apply(self, _: #name) -> malvolio::prelude::#segment {
                                let string: std::borrow::Cow<'static, str> =
                                    if let Some(x) = self.read_attribute("style") {
                                        format!("{} {}", x, #tokens).into()
                                    } else {
                                        #tokens.into()
                                    };
                                self.attribute(::malvolio::prelude::Style::new(string))
                            }
                        }
                    }
                })
                .fold(quote! {}, |a, b| quote! {#a #b})
        }
    }
}

struct CssPropsInnerStyleOutputter<'a>(pub &'a CssPropsInner);

impl<'a> std::fmt::Display for CssPropsInnerStyleOutputter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(value) = &self.0.binding {
            f.write_str("binding:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.width {
            f.write_str("width:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.list_style_image {
            f.write_str("list-style-image:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.position {
            f.write_str("position:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.overflow {
            f.write_str("overflow:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.unicode_bidi {
            f.write_str("unicode-bidi:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.marker_mid {
            f.write_str("marker-mid:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.line_height {
            f.write_str("line-height:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.cue_before {
            f.write_str("cue-before:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.caption_side {
            f.write_str("caption-side:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_right_color {
            f.write_str("border-right-color:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.orphans {
            f.write_str("orphans:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.list_style_type {
            f.write_str("list-style-type:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.font_size_adjust {
            f.write_str("font-size_adjust:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.page_break_inside {
            f.write_str("page-break-inside:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_top_width {
            f.write_str("border-top-width:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_top_style {
            f.write_str("border-top-style:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.max_width {
            f.write_str("max-width:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.voice_family {
            f.write_str("voice-family:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_style {
            f.write_str("border-style:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.speak {
            f.write_str("speak:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.azimuth {
            f.write_str("azimuth:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.color {
            f.write_str("color:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.direction {
            f.write_str("direction:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.flex_direction {
            f.write_str("flex-direction:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.alignment_baseline {
            f.write_str("alignment-baseline:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_left {
            f.write_str("border-left:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.text_align {
            f.write_str("text-align:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_bottom_width {
            f.write_str("border-bottom-width:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_bottom {
            f.write_str("border-bottom:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.min_width {
            f.write_str("min-width:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.margin_left {
            f.write_str("margin-left:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.pause_before {
            f.write_str("pause-before:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.speak_numeral {
            f.write_str("speak-numeral:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.pitch_range {
            f.write_str("pitch-range:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.property_name {
            f.write_str("property-name:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.font_stretch {
            f.write_str("font-stretch:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.padding_top {
            f.write_str("padding-top:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.margin {
            f.write_str("margin:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.empty_cells {
            f.write_str("empty-cells:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.cursor {
            f.write_str("cursor:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_top {
            f.write_str("border-top:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.dominant_baseline {
            f.write_str("dominant-baseline:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_right {
            f.write_str("border-right:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.min_height {
            f.write_str("min-height:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.padding_bottom {
            f.write_str("padding-bottom:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_collapse {
            f.write_str("border-collapse:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.bottom {
            f.write_str("bottom:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.background_color {
            f.write_str("background-color:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.text_shadow {
            f.write_str("text-shadow:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.content {
            f.write_str("content:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_spacing {
            f.write_str("border-spacing:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.quotes {
            f.write_str("quotes:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.speak_header {
            f.write_str("speak-header:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.counter_reset {
            f.write_str("counter-reset:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.clear {
            f.write_str("clear:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.table_layout {
            f.write_str("table-layout:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_left_style {
            f.write_str("border-left-style:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.baseline_shift {
            f.write_str("baseline-shift:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.height {
            f.write_str("height:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.word_spacing {
            f.write_str("word-spacing:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_left_width {
            f.write_str("border-left-width:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.z_index {
            f.write_str("z-index:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.marker_end {
            f.write_str("marker-end:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.letter_spacing {
            f.write_str("letter-spacing:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_left_color {
            f.write_str("border-left-color:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.display {
            f.write_str("display:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_right_style {
            f.write_str("border-right-style:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.font {
            f.write_str("font:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.page_break_before {
            f.write_str("page-break-before:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.text_transform {
            f.write_str("text-transform:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.richness {
            f.write_str("richness:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.background_repeat {
            f.write_str("background-repeat:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.float {
            f.write_str("float:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.white_space {
            f.write_str("white-space:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_bottom_color {
            f.write_str("border-bottom-color:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.play_during {
            f.write_str("play-during:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.cue {
            f.write_str("cue:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.outline_style {
            f.write_str("outline-style:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.outline {
            f.write_str("outline:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_top_color {
            f.write_str("border-top-color:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.font_size {
            f.write_str("font-size:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.name {
            f.write_str("name:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.counter_increment {
            f.write_str("counter-increment:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.volume {
            f.write_str("volume:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.vertical_align {
            f.write_str("vertical-align:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.padding {
            f.write_str("padding:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.speak_punctuation {
            f.write_str("speak-punctuation:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.glyph_orientation_horizontal {
            f.write_str("glyph-orientation-horizontal:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.marker_offset {
            f.write_str("marker-offset:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.margin_top {
            f.write_str("margin-top:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.left {
            f.write_str("left:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.page_break_after {
            f.write_str("page-break-after:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_bottom_style {
            f.write_str("border-bottom-style:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.margin_bottom {
            f.write_str("margin-bottom:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.text_indent {
            f.write_str("text-indent:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_width {
            f.write_str("border-width:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.pause_after {
            f.write_str("pause-after:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.rendering_intent {
            f.write_str("rendering-intent:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.margin_right {
            f.write_str("margin-right:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.clip {
            f.write_str("clip:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.pitch {
            f.write_str("pitch:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.list_style_position {
            f.write_str("list-style-position:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border {
            f.write_str("border:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.background {
            f.write_str("background:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.speech_rate {
            f.write_str("speech-rate:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.kerning {
            f.write_str("kerning:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.font_style {
            f.write_str("font-style:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.top {
            f.write_str("top:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.cue_after {
            f.write_str("cue_after:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.background_attachment {
            f.write_str("background_attachment:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.right {
            f.write_str("right:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.max_height {
            f.write_str("max_height:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.font_family {
            f.write_str("font-family:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.background_image {
            f.write_str("background_image:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.background_position {
            f.write_str("background_position:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_right_width {
            f.write_str("border-right-width:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.text_anchor {
            f.write_str("text-anchor:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.writing_mode {
            f.write_str("writing_mode:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.marker_start {
            f.write_str("marker-start:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.glyph_orientation_vertical {
            f.write_str("glyph-orientation-vertical:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.padding_right {
            f.write_str("padding-right:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.src {
            f.write_str("src:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.visibility {
            f.write_str("visibility:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.outline_color {
            f.write_str("outline-color:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.font_variant {
            f.write_str("font-variant:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.text_decoration {
            f.write_str("text-decoration:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.stress {
            f.write_str("stress:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.widows {
            f.write_str("widows:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.outline_width {
            f.write_str("outline-width:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_color {
            f.write_str("border-color:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.font_weight {
            f.write_str("font-weight:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.pause {
            f.write_str("pause:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.marker {
            f.write_str("marker:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.padding_left {
            f.write_str("padding-left:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.list_style {
            f.write_str("list-style:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.elevation {
            f.write_str("elevation:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        Ok(())
    }
}

struct CssPropsInnerClassOutputter<'a>(pub &'a CssPropsInner, pub String);

impl<'a> std::fmt::Display for CssPropsInnerClassOutputter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(".")?;
        f.write_str(&self.1)?;
        f.write_str("{")?;
        if let Some(value) = &self.0.binding {
            f.write_str("binding:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.width {
            f.write_str("width:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.list_style_image {
            f.write_str("list-style-image:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.position {
            f.write_str("position:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.overflow {
            f.write_str("overflow:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.unicode_bidi {
            f.write_str("unicode-bidi:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.marker_mid {
            f.write_str("marker-mid:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.line_height {
            f.write_str("line-height:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.cue_before {
            f.write_str("cue-before:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.caption_side {
            f.write_str("caption-side:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_right_color {
            f.write_str("border-right-color:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.orphans {
            f.write_str("orphans:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.list_style_type {
            f.write_str("list-style-type:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.font_size_adjust {
            f.write_str("font-size_adjust:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.page_break_inside {
            f.write_str("page-break-inside:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_top_width {
            f.write_str("border-top-width:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_top_style {
            f.write_str("border-top-style:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.max_width {
            f.write_str("max-width:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.voice_family {
            f.write_str("voice-family:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_style {
            f.write_str("border-style:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.speak {
            f.write_str("speak:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.azimuth {
            f.write_str("azimuth:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.color {
            f.write_str("color:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.direction {
            f.write_str("direction:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.alignment_baseline {
            f.write_str("alignment-baseline:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_left {
            f.write_str("border-left:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.text_align {
            f.write_str("text-align:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_bottom_width {
            f.write_str("border-bottom-width:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_bottom {
            f.write_str("border-bottom:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.min_width {
            f.write_str("min-width:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.margin_left {
            f.write_str("margin-left:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.pause_before {
            f.write_str("pause-before:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.speak_numeral {
            f.write_str("speak-numeral:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.pitch_range {
            f.write_str("pitch-range:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.property_name {
            f.write_str("property-name:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.font_stretch {
            f.write_str("font-stretch:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.padding_top {
            f.write_str("padding-top:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.margin {
            f.write_str("margin:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.empty_cells {
            f.write_str("empty-cells:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.cursor {
            f.write_str("cursor:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_top {
            f.write_str("border-top:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.dominant_baseline {
            f.write_str("dominant-baseline:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_right {
            f.write_str("border-right:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.min_height {
            f.write_str("min-height:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.padding_bottom {
            f.write_str("padding-bottom:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_collapse {
            f.write_str("border-collapse:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.bottom {
            f.write_str("bottom:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.background_color {
            f.write_str("background-color:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.text_shadow {
            f.write_str("text-shadow:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.content {
            f.write_str("content:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_spacing {
            f.write_str("border-spacing:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.quotes {
            f.write_str("quotes:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.speak_header {
            f.write_str("speak-header:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.counter_reset {
            f.write_str("counter-reset:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.clear {
            f.write_str("clear:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.table_layout {
            f.write_str("table-layout:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_left_style {
            f.write_str("border-left-style:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.baseline_shift {
            f.write_str("baseline-shift:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.height {
            f.write_str("height:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.word_spacing {
            f.write_str("word-spacing:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_left_width {
            f.write_str("border-left-width:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.z_index {
            f.write_str("z-index:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.marker_end {
            f.write_str("marker-end:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.letter_spacing {
            f.write_str("letter-spacing:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_left_color {
            f.write_str("border-left-color:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.display {
            f.write_str("display:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_right_style {
            f.write_str("border-right-style:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.font {
            f.write_str("font:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.page_break_before {
            f.write_str("page-break-before:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.text_transform {
            f.write_str("text-transform:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.richness {
            f.write_str("richness:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.background_repeat {
            f.write_str("background-repeat:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.float {
            f.write_str("float:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.white_space {
            f.write_str("white-space:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_bottom_color {
            f.write_str("border-bottom-color:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.play_during {
            f.write_str("play-during:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.cue {
            f.write_str("cue:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.outline_style {
            f.write_str("outline-style:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.outline {
            f.write_str("outline:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_top_color {
            f.write_str("border-top-color:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.font_size {
            f.write_str("font-size:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.name {
            f.write_str("name:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.counter_increment {
            f.write_str("counter-increment:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.volume {
            f.write_str("volume:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.vertical_align {
            f.write_str("vertical-align:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.padding {
            f.write_str("padding:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.speak_punctuation {
            f.write_str("speak-punctuation:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.glyph_orientation_horizontal {
            f.write_str("glyph-orientation-horizontal:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.marker_offset {
            f.write_str("marker-offset:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.margin_top {
            f.write_str("margin-top:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.left {
            f.write_str("left:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.page_break_after {
            f.write_str("page-break-after:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_bottom_style {
            f.write_str("border-bottom-style:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.margin_bottom {
            f.write_str("margin-bottom:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.text_indent {
            f.write_str("text-indent:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_width {
            f.write_str("border-width:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.pause_after {
            f.write_str("pause-after:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.rendering_intent {
            f.write_str("rendering-intent:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.margin_right {
            f.write_str("margin-right:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.clip {
            f.write_str("clip:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.pitch {
            f.write_str("pitch:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.list_style_position {
            f.write_str("list-style-position:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border {
            f.write_str("border:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.background {
            f.write_str("background:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.speech_rate {
            f.write_str("speech-rate:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.kerning {
            f.write_str("kerning:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.font_style {
            f.write_str("font-style:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.top {
            f.write_str("top:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.cue_after {
            f.write_str("cue_after:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.background_attachment {
            f.write_str("background_attachment:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.right {
            f.write_str("right:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.max_height {
            f.write_str("max_height:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.font_family {
            f.write_str("font-family:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.background_image {
            f.write_str("background_image:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.background_position {
            f.write_str("background_position:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_right_width {
            f.write_str("border-right-width:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.text_anchor {
            f.write_str("text-anchor:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.writing_mode {
            f.write_str("writing_mode:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.marker_start {
            f.write_str("marker-start:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.glyph_orientation_vertical {
            f.write_str("glyph-orientation-vertical:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.padding_right {
            f.write_str("padding-right:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.src {
            f.write_str("src:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.visibility {
            f.write_str("visibility:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.outline_color {
            f.write_str("outline-color:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.font_variant {
            f.write_str("font-variant:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.text_decoration {
            f.write_str("text-decoration:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.stress {
            f.write_str("stress:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.widows {
            f.write_str("widows:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.outline_width {
            f.write_str("outline-width:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.border_color {
            f.write_str("border-color:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.font_weight {
            f.write_str("font-weight:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.pause {
            f.write_str("pause:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.marker {
            f.write_str("marker:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.padding_left {
            f.write_str("padding-left:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.list_style {
            f.write_str("list-style:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        if let Some(value) = &self.0.elevation {
            f.write_str("elevation:")?;
            f.write_str(&value)?;
            f.write_str(";")?;
        }
        f.write_str("}")
    }
}
