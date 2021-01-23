use std::{borrow::Cow, collections::HashMap, fmt::Display};

#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;
use crate::{
    attributes::IntoAttribute, into_attribute_for_grouping_enum, into_grouping_union, utility_enum,
};

use super::head::head_node::HeadNode;

#[derive(Default, Debug, Clone)]
/// A metadata element. Useful for adding metadata which can not be represented through other HTML
/// tags.
///
/// See the [MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meta) for more
/// info.
pub struct Meta {
    attrs: HashMap<&'static str, Cow<'static, str>>,
}

impl Meta {
    #[inline(always)]
    /// Add an attribute to this meta tag.
    pub fn attribute<A>(mut self, attr: A) -> Self
    where
        A: Into<MetaAttr>,
    {
        let (a, b) = attr.into().into_attribute();
        self.attrs.insert(a, b);
        self
    }
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Meta {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("meta");
        for (a, b) in self.attrs {
            vtag.add_attribute(a, &b.to_string())
        }
        vtag.into()
    }
}

impl Display for Meta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<meta")?;
        for attr in &self.attrs {
            f.write_str(" ")?;
            attr.0.fmt(f)?;
            f.write_str("=\"")?;
            attr.1.fmt(f)?;
            f.write_str("\"")?;
        }
        f.write_str("/>")
    }
}

into_grouping_union!(Meta, HeadNode);

utility_enum!(
    pub enum MetaAttr {
        Content(Content),
        MetaName(MetaName),
    }
);

into_attribute_for_grouping_enum!(MetaAttr, Content, MetaName);

/// The "name" attribute for meta tags. This is called `MetaName` to disambiguate it from other
/// tags.
pub enum MetaName {
    Charset,
}

impl IntoAttribute for MetaName {
    fn into_attribute(self) -> (&'static str, Cow<'static, str>) {
        (
            "name",
            match self {
                MetaName::Charset => "charset",
            }
            .into(),
        )
    }
}

into_grouping_union!(MetaName, MetaAttr);

pub struct Content(Cow<'static, str>);

impl Content {
    /// Create a new "content" attribute, which can then be applied to a meta tag.
    pub fn new<C>(c: C) -> Self
    where
        C: Into<Cow<'static, str>>,
    {
        Self(c.into())
    }
}

impl IntoAttribute for Content {
    fn into_attribute(self) -> (&'static str, Cow<'static, str>) {
        ("content", self.0)
    }
}

into_grouping_union!(Content, MetaAttr);

#[cfg(test)]
mod test {
    use crate::prelude::*;
    #[test]
    fn test_a_with_attributes() {
        let document = Meta::default()
            .attribute(MetaName::Charset)
            .attribute(Content::new("utf-8"))
            .to_string();
        let document = scraper::Html::parse_document(&document);
        let a = scraper::Selector::parse("meta").unwrap();
        let a = document.select(&a).next().unwrap().value();
        assert_eq!(a.attr("name").unwrap(), "charset");
    }
}
