use std::{borrow::Cow, collections::HashMap, fmt::Display};

use crate::{
    attributes::{common::Class, IntoAttribute},
    prelude::Style,
};

#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;
use crate::{
    into_attribute_for_grouping_enum, into_grouping_union, prelude::Id, to_html, utility_enum,
};

use super::body::body_node::BodyNode;

#[derive(Debug, Derivative, Clone)]
#[derivative(Default(new = "true"))]
pub struct Div {
    children: Vec<BodyNode>,
    attrs: HashMap<&'static str, Cow<'static, str>>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Div {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("div");
        vtag.add_children(self.children.into_iter().map(IntoVNode::into));
        for (a, b) in self.attrs {
            vtag.add_attribute(a, &b.to_string())
        }
        vtag.into()
    }
}

impl Div {
    pub fn children<C, D>(mut self, children: C) -> Self
    where
        C: IntoIterator<Item = D>,
        D: Into<BodyNode>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<BodyNode>,
    {
        self.children.push(child.into());
        self
    }
    pub fn map<F>(mut self, mapping: F) -> Self
    where
        F: Fn(Self) -> Self,
    {
        self = mapping(self);
        self
    }
    pub fn attribute<A>(mut self, attribute: A) -> Self
    where
        A: IntoAttribute,
    {
        let (a, b) = attribute.into_attribute();
        self.attrs.insert(a, b);
        self
    }
    to_html!();
}

impl Display for Div {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<div")?;
        for attr in &self.attrs {
            f.write_str(" ")?;
            attr.0.fmt(f)?;
            f.write_str("=\"")?;
            attr.1.fmt(f)?;
            f.write_str("\"")?;
        }
        f.write_str("/>")?;
        for node in &self.children {
            node.fmt(f)?;
        }
        f.write_str("</div>")
    }
}
into_grouping_union!(Div, BodyNode);

utility_enum!(
    pub enum DivAttr {
        Id(Id),
        Class(Class),
        Style(Style),
    }
);

into_attribute_for_grouping_enum!(DivAttr, Id, Class, Style);

into_grouping_union!(Id, DivAttr);

into_grouping_union!(Class, DivAttr);

into_grouping_union!(Style, DivAttr);

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    #[test]
    fn test_div_attributes() {
        let document = Div::default()
            .attribute(crate::prelude::Class::from("some-class"))
            .attribute(crate::prelude::Style::new("font-family: Arial;"))
            .to_string();
        let document = scraper::Html::parse_document(&document);
        let div_selector = scraper::Selector::parse("div").unwrap();
        assert_eq!(document.select(&div_selector).collect::<Vec<_>>().len(), 1);
        let div = document.select(&div_selector).next().unwrap();
        assert_eq!(div.value().attr("class").unwrap(), "some-class");
        assert_eq!(div.value().attr("style").unwrap(), "font-family: Arial;");
    }
    #[test]
    fn test_div_children() {
        let document = Div::default()
            .children(
                vec!["1", "2", "3"]
                    .into_iter()
                    .map(|string| P::with_text(string)),
            )
            .to_string();
        let document = scraper::Html::parse_document(&document);
        let div_selector = scraper::Selector::parse("div").unwrap();
        let div = document.select(&div_selector).next().unwrap();
        let children = div.children().collect::<Vec<_>>();
        assert_eq!(
            children[0]
                .children()
                .next()
                .unwrap()
                .value()
                .as_text()
                .unwrap()
                .to_string()
                .as_str(),
            "1"
        );
        assert_eq!(
            children[1]
                .children()
                .next()
                .unwrap()
                .value()
                .as_text()
                .unwrap()
                .to_string()
                .as_str(),
            "2"
        );
        assert_eq!(
            children[2]
                .children()
                .next()
                .unwrap()
                .value()
                .as_text()
                .unwrap()
                .to_string()
                .as_str(),
            "3"
        );
    }
}
