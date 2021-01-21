use std::{collections::HashMap, fmt::Display};

use crate::{impl_of_data_struct_insert, into_grouping_union, utils::write_attributes};

#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;
#[cfg(feature = "with_yew")]
use crate::utils::write_attributes_to_vtag;
#[cfg(feature = "with_yew")]
use std::rc::Rc;
#[cfg(feature = "with_yew")]
use yew::virtual_dom::Listener;

use super::{body::body_node::BodyNode, option::SelectOption};

#[derive(Default, Debug, Clone)]
pub struct Select {
    attrs: HashMap<&'static str, String>,
    children: Vec<SelectOption>,
}

impl Select {
    impl_of_data_struct_insert!();
    pub fn children<I, C>(mut self, children: I) -> Self
    where
        C: Into<SelectOption>,
        I: IntoIterator<Item = C>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<SelectOption>,
    {
        self.children.push(child.into());
        self
    }
}

into_grouping_union!(Select, BodyNode);

impl Display for Select {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<select ")?;
        write_attributes(&self.attrs, f)?;
        f.write_str(">")?;
        for child in &self.children {
            child.fmt(f)?;
        }
        f.write_str("</select>")
    }
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Select {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("select");
        write_attributes_to_vtag(self.attrs, &mut vtag);
        vtag.add_children(self.children.into_iter().map(IntoVNode::into));
        vtag.into()
    }
}
