/*
This source code file is distributed subject to the terms of the Mozilla Public License v2.0.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
use std::fmt::Display;

use self::body_node::BodyNode;
#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;

/// Contains the `BodyNode` enum.
pub mod body_node;

#[derive(Derivative, Debug, Clone)]
#[derivative(Default(new = "true"))]
/// The <body> tag.
pub struct Body {
    children: Vec<BodyNode>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Body {
    fn into_vnode(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("body");
        vtag.add_children(self.children.into_iter().map(IntoVNode::into_vnode));
        vtag.into()
    }
}

impl Body {
    /// Attach multiple children to this tag, from an iterator of items implementing
    /// `Into<BodyNode>`
    pub fn children<I, C>(mut self, children: I) -> Self
    where
        C: Into<BodyNode>,
        I: IntoIterator<Item = C>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
    /// Attach a single child to this tag.
    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<BodyNode>,
    {
        self.children.push(child.into());
        self
    }
    /// Apply a function to this tag.
    pub fn map<F>(self, mapping: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        mapping(self)
    }
}

impl Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<body>")?;
        for node in &self.children {
            node.fmt(f)?;
        }
        f.write_str("</body>")
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    #[test]
    fn test_children() {
        use crate::prelude::*;
        let document = Body::new()
            .children(
                vec!["1", "2", "3"]
                    .into_iter()
                    .map(|item| H1::new(item).attribute(Id::new(item))),
            )
            .to_string();
        let document = scraper::Html::parse_document(&document);
        let h1_selector = scraper::Selector::parse("h1").unwrap();
        let h1s = document.select(&h1_selector).collect::<Vec<_>>();
        assert_eq!(h1s.len(), 3);
        assert_eq!(h1s[0].value().attr("id"), Some("1"));
        assert_eq!(
            h1s[0]
                .first_child()
                .unwrap()
                .value()
                .as_text()
                .map(Deref::deref),
            Some("1")
        );
        assert_eq!(h1s[1].value().attr("id"), Some("2"));
        assert_eq!(
            h1s[1]
                .first_child()
                .unwrap()
                .value()
                .as_text()
                .map(Deref::deref),
            Some("2")
        );
        assert_eq!(h1s[2].value().attr("id"), Some("3"));
        assert_eq!(
            h1s[2]
                .first_child()
                .unwrap()
                .value()
                .as_text()
                .map(Deref::deref),
            Some("3")
        );
    }
}
