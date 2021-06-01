use crate::into_grouping_union;
use crate::tags::head::head_node::HeadNode;
use std::{borrow::Cow, fmt::Display};

/// The `<style>` tag, useful for embedding CSS styling inside HTML documents.
///
/// See [MDN's page on this](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meta) for
/// further information.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "pub_fields", derive(FieldsAccessibleVariant))]
pub struct StyleTag {
    text: Cow<'static, str>,
}

/// Creates a new `Style` tag – functionally equivalent to `Style::new()` (but easier to type.)
pub fn style(text: impl Into<Cow<'static, str>>) -> StyleTag {
    StyleTag::new(text)
}

impl StyleTag {
    /// Create a new style tag.
    pub fn new<C>(c: C) -> Self
    where
        C: Into<Cow<'static, str>>,
    {
        Self { text: c.into() }
    }
}

impl Display for StyleTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<style>")?;
        f.write_str(&self.text)?;
        f.write_str("</style>")
    }
}

#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
impl crate::into_vnode::IntoVNode for StyleTag {
    fn into_vnode(self) -> yew::virtual_dom::VNode {
        let mut tag = ::yew::virtual_dom::VTag::new("style");
        tag.add_child(::yew::virtual_dom::VText::new(self.text.to_string()).into());
        tag.into()
    }
}

into_grouping_union!(StyleTag, HeadNode);
