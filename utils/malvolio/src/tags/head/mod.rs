/*
This source code file is distributed subject to the terms of the Mozilla Public License v2.0.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

use std::fmt::Display;

use self::head_node::HeadNode;
#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
use crate::into_vnode::IntoVNode;

/// Items which can be mounted to head.
pub mod head_node;

#[derive(Derivative, Debug, Clone)]
#[derivative(Default = "new")]
/// The <head> tag.
pub struct Head {
    children: Vec<HeadNode>,
}

#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
impl IntoVNode for Head {
    fn into_vnode(self) -> yew::virtual_dom::VNode {
        let mut tag = yew::virtual_dom::VTag::new("head");
        tag.add_children(self.children.into_iter().map(IntoVNode::into_vnode));
        tag.into()
    }
}

impl Head {
    /// Add a number of children to this <head> tag from an iterator.
    pub fn children<I, C>(mut self, children: I) -> Self
    where
        C: Into<HeadNode>,
        I: IntoIterator<Item = C>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
    /// Add a single child to this <head> tag.
    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<HeadNode>,
    {
        self.children.push(child.into());
        self
    }
}

impl Display for Head {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<head>")?;
        for child in &self.children {
            child.fmt(f)?;
        }
        f.write_str("</head>")
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_children() {
        let document = Head::new()
            .children(vec!["1", "2", "3", "4"].into_iter().map(|item| {
                Meta::new()
                    .attribute(MetaName::Charset)
                    .attribute(Content::new(item))
            }))
            .to_string();
        let document = scraper::Html::parse_document(&document);
        let selector = scraper::Selector::parse("meta").unwrap();
        let res = document.select(&selector).collect::<Vec<_>>();
        assert_eq!(res.len(), 4);
        assert_eq!(res[0].value().attr("content"), Some("1"));
        assert_eq!(res[1].value().attr("content"), Some("2"));
        assert_eq!(res[2].value().attr("content"), Some("3"));
        assert_eq!(res[3].value().attr("content"), Some("4"));
    }
}
