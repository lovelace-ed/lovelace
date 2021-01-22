use std::fmt::Display;

#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;
use crate::{add_single_attribute, into_grouping_union, to_html};

use super::body::body_node::BodyNode;

#[derive(Debug, Clone, Default)]
pub struct Div {
    children: Vec<BodyNode>,
    attrs: Vec<(&'static str, &'static str)>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Div {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("div");
        vtag.add_children(self.children.into_iter().map(IntoVNode::into));
        for (a, b) in self.attrs {
            vtag.add_attribute(a, b.to_string())
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
    add_single_attribute!('static);
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    #[test]
    fn test_div_attributes() {
        let document = Div::default()
            .attribute("class", "some-class")
            .attribute("style", "font-family: Arial;")
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
