use std::fmt::Display;

use crate::into_grouping_union;
#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;

use super::head::head_node::HeadNode;

#[derive(Default, Debug, Clone)]
pub struct Meta {
    attrs: Vec<(&'static str, &'static str)>,
}

impl Meta {
    #[inline(always)]
    pub fn attribute(mut self, k: &'static str, v: &'static str) -> Self {
        self.attrs.push((k, v));
        self
    }
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Meta {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("meta");
        for (a, b) in self.attrs {
            vtag.add_attribute(a, b.to_string())
        }
        vtag.into()
    }
}

impl Display for Meta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<meta")?;
        for attr in &self.attrs {
            f.write_str(" ")?;
            attr.0.fmt(f)?;
            f.write_str("=\"")?;
            attr.1.fmt(f)?;
            f.write_str("\"")?;
        }
        f.write_str("/>")
    }
}

into_grouping_union!(Meta, HeadNode);
