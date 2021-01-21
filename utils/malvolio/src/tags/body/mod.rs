use std::fmt::Display;

use self::body_node::BodyNode;
#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;

pub mod body_node;

#[derive(Default, Debug, Clone)]
pub struct Body {
    children: Vec<BodyNode>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Body {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("body");
        vtag.add_children(self.children.into_iter().map(IntoVNode::into));
        vtag.into()
    }
}

impl Body {
    pub fn children<I, C>(mut self, children: I) -> Self
    where
        C: Into<BodyNode>,
        I: IntoIterator<Item = C>,
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
