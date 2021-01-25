use std::fmt::Display;

use self::head_node::HeadNode;
#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;

/// Items which can be mounted to head.
pub mod head_node;

#[derive(Default, Debug, Clone)]
/// The <head> tag.
pub struct Head {
    children: Vec<HeadNode>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Head {
    fn into_vnode(self) -> yew::virtual_dom::VNode {
        let mut tag = yew::virtual_dom::VTag::new("head");
        tag.add_children(self.children.into_iter().map(IntoVNode::into_vnode));
        tag.into()
    }
}

impl Head {
    /// Add a number of children to this <head> tag from an iterator.
    pub fn children<C>(mut self, children: Vec<C>) -> Self
    where
        C: Into<HeadNode>,
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
