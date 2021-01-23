use crate::impl_of_data_struct_insert;
#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;
use crate::utils::write_attributes;
#[cfg(feature = "with_yew")]
use crate::utils::write_attributes_to_vtag;
use ammonia::clean;
use std::{borrow::Cow, collections::HashMap, fmt::Display};

#[derive(Default, Debug, Clone)]
pub struct SelectOption {
    attrs: HashMap<&'static str, String>,
    text: Cow<'static, str>,
}

impl SelectOption {
    impl_of_data_struct_insert!();
    /// Adds the supplied text to this node, overwriting the previously existing text (if text has
    /// already been added to the node).
    ///
    /// This method sanitises the input (i.e. it escapes HTML);
    /// this might not be what you want – if you are *absolutely certain* that the text you are
    /// providing does not come from a potentially malicious source (e.g. user-supplied text can
    /// contain script tags which will execute unwanted code) you can use `text_unsanitized` which
    /// is identical to this method, except for that it does not sanitise the inputted text (and is
    /// thus slightly faster).
    pub fn text<S>(mut self, text: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.text = clean(&text.into()).into();
        self
    }
    /// Adds the supplied text to this node, overwriting the previously existing text (if text has
    /// already been added to the node).
    ///
    /// WARNING: Do not (under any circumstances) use this method with unescaped user-supplied text.
    /// It will be rendered and poses a major security threat to your application. If in doubt, use
    /// the `text` method instead of this one (the risk is much lower that way).
    pub fn text_unsanitized<S>(mut self, text: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.text = text.into();
        self
    }
}

impl Display for SelectOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<option ")?;
        write_attributes(&self.attrs, f)?;
        f.write_str(">")?;
        self.text.fmt(f)?;
        f.write_str("</option>")
    }
}

#[cfg(feature = "with_yew")]
impl IntoVNode for SelectOption {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("option");
        write_attributes_to_vtag(self.attrs, &mut vtag);
        vtag.add_child(::yew::virtual_dom::VText::new(self.text.to_string()).into());
        vtag.into()
    }
}
