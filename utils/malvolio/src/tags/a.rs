#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;
use crate::{
    attributes::IntoAttribute, into_attribute_for_grouping_enum, into_grouping_union, prelude::Id,
    to_html, utility_enum,
};
#[cfg(feature = "with_yew")]
use std::rc::Rc;
use std::{borrow::Cow, collections::HashMap, fmt::Display};
#[cfg(feature = "with_yew")]
use yew::virtual_dom::Listener;

use super::body::body_node::BodyNode;

#[derive(Debug, Clone, Default)]
pub struct A {
    attrs: HashMap<&'static str, Cow<'static, str>>,
    text: Cow<'static, str>,
    #[cfg(feature = "with_yew")]
    listeners: Vec<Rc<dyn Listener>>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for A {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vnode = yew::virtual_dom::VTag::new("a");
        for (a, b) in self.attrs {
            vnode.add_attribute(a, &b.to_string())
        }
        vnode.add_child(yew::virtual_dom::VText::new(String::from(self.text)).into());
        vnode.into()
    }
}

impl A {
    pub fn text<S>(mut self, text: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.text = text.into();
        self
    }
    pub fn attribute<I>(mut self, attribute: I) -> Self
    where
        I: Into<AAttr>,
    {
        let res = attribute.into().into_attribute();
        self.attrs.insert(res.0, res.1);
        self
    }
    to_html!();
    #[cfg(feature = "with_yew")]
    pub fn listener(mut self, listener: Rc<dyn Listener>) -> Self {
        self.listeners.push(listener);
        self
    }
}

impl Display for A {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<a")?;
        for attr in &self.attrs {
            f.write_str(" ")?;
            attr.0.fmt(f)?;
            f.write_str("=\"")?;
            attr.1.fmt(f)?;
            f.write_str("\"")?;
        }
        f.write_str("\"")?;
        f.write_str(">")?;
        self.text.fmt(f)?;
        f.write_str("</a>")
    }
}
into_grouping_union!(A, BodyNode);

utility_enum!(
    pub enum AAttr {
        Href(Href),
        Download(Download),
        Target(Target),
        Id(Id),
    }
);

into_grouping_union!(Id, AAttr);

into_attribute_for_grouping_enum!(AAttr, Href, Download, Target, Id);

pub struct Href(Cow<'static, str>);

impl Href {
    pub fn new<C>(value: C) -> Self
    where
        C: Into<Cow<'static, str>>,
    {
        Self(value.into())
    }
}
into_grouping_union!(Href, AAttr);

pub struct Download(Cow<'static, str>);

impl Download {
    pub fn new<C>(value: C) -> Self
    where
        C: Into<Cow<'static, str>>,
    {
        Self(value.into())
    }
}

impl IntoAttribute for Download {
    fn into_attribute(self) -> (&'static str, Cow<'static, str>) {
        ("download", self.0)
    }
}

into_grouping_union!(Download, AAttr);

impl IntoAttribute for Href {
    fn into_attribute(self) -> (&'static str, Cow<'static, str>) {
        ("href", self.0)
    }
}

pub enum Target {
    Blank,
}

into_grouping_union!(Target, AAttr);

impl IntoAttribute for Target {
    fn into_attribute(self) -> (&'static str, Cow<'static, str>) {
        (
            "target",
            match self {
                Target::Blank => "_blank".into(),
            },
        )
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    #[test]
    fn test_a_with_attributes() {
        let document = A::default()
            .attribute(super::Href::new("https://example.com"))
            .attribute(super::Target::Blank)
            .attribute(super::Download::new("some-download"))
            .to_string();
        let document = scraper::Html::parse_document(&document);
        let a = scraper::Selector::parse("a").unwrap();
        let a = document.select(&a).next().unwrap().value();
        assert_eq!(a.attr("href").unwrap(), "https://example.com");
        assert_eq!(a.attr("target").unwrap(), "_blank");
        assert_eq!(a.attr("download").unwrap(), "some-download");
    }
}
