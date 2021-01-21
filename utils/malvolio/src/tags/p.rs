use std::{borrow::Cow, fmt::Display};

#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;
use crate::{into_grouping_union, text::Text};

use super::body::body_node::BodyNode;

#[derive(Default, Debug, Clone)]
pub struct P {
    children: Vec<BodyNode>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for P {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("br");
        vtag.add_children(self.children.into_iter().map(IntoVNode::into));
        vtag.into()
    }
}

into_grouping_union!(P, BodyNode);

impl Display for P {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<p>")?;
        for child in &self.children {
            child.fmt(f)?;
        }
        f.write_str("</p>")
    }
}

impl P {
    pub fn children<C>(mut self, children: Vec<C>) -> Self
    where
        C: Into<BodyNode>,
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
    pub fn with_text<S>(text: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        P::default().child(BodyNode::Text(Text::new(text)))
    }
    pub fn text<S>(self, text: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.child(BodyNode::Text(Text::new(text)))
    }
}
