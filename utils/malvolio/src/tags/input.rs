use crate::into_grouping_union;
#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;
#[cfg(feature = "with_yew")]
use std::rc::Rc;
use std::{borrow::Cow, fmt::Display};
#[cfg(feature = "with_yew")]
use yew::virtual_dom::Listener;

use super::body::body_node::BodyNode;

#[derive(Debug, Clone, Default)]
pub struct Input {
    attrs: Vec<(&'static str, Cow<'static, str>)>,
    #[cfg(feature = "with_yew")]
    listeners: Vec<Rc<dyn Listener>>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Input {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("input");
        self.attrs
            .clone()
            .iter()
            .find(|item| item.0 == "type")
            .map(|(_, res)| vtag.set_kind(&res.to_string()));
        for (a, b) in self.attrs {
            vtag.add_attribute(a, &b)
        }
        vtag.add_listeners(self.listeners.clone());
        vtag.into()
    }
}

impl Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<input")?;
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

into_grouping_union!(Input, BodyNode);

impl Input {
    #[inline(always)]
    pub fn attribute<S1, S2>(mut self, k: S1, v: S2) -> Self
    where
        S1: Into<&'static str>,
        S2: Into<Cow<'static, str>>,
    {
        self.attrs.push((k.into(), v.into()));
        self
    }
    #[cfg(feature = "with_yew")]
    pub fn listener(mut self, listener: Rc<dyn Listener>) -> Self {
        self.listeners.push(listener);
        self
    }
    #[cfg(feature = "with_yew")]
    pub fn listeners(mut self, listeners: Vec<Rc<dyn Listener>>) -> Self {
        self.listeners.extend(listeners);
        self
    }
}
