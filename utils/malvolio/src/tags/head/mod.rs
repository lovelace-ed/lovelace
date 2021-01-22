use std::fmt::Display;

#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;

use self::head_node::HeadNode;

pub mod head_node;

#[derive(Default, Debug, Clone)]
pub struct Head {
    children: Vec<HeadNode>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Head {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut tag = yew::virtual_dom::VTag::new("head");
        tag.add_children(self.children.into_iter().map(IntoVNode::into));
        tag.into()
    }
}

impl Head {
    pub fn children<C>(mut self, children: Vec<C>) -> Self
    where
        C: Into<HeadNode>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
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
