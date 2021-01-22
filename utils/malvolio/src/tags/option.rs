use crate::impl_of_data_struct_insert;
#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;
use crate::utils::write_attributes;
#[cfg(feature = "with_yew")]
use crate::utils::write_attributes_to_vtag;
use std::{borrow::Cow, collections::HashMap, fmt::Display};
#[derive(Default, Debug, Clone)]
pub struct SelectOption {
    attrs: HashMap<&'static str, String>,
    text: Cow<'static, str>,
}

impl SelectOption {
    impl_of_data_struct_insert!();
    pub fn text<S>(mut self, text: S) -> Self
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
        vtag.add_child(::yew::virtual_dom::VText::new(self.text).into());
        vtag.into()
    }
}
