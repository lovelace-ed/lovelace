use std::fmt::Display;

#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;
use crate::{add_single_attribute, into_grouping_union, to_html};
#[cfg(feature = "with_yew")]
use yew::virtual_dom::Listener;

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
