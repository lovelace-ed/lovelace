#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;
use crate::{add_single_attribute, into_grouping_union, to_html};
#[cfg(feature = "with_yew")]
use std::rc::Rc;
use std::{borrow::Cow, fmt::Display};
#[cfg(feature = "with_yew")]
use yew::virtual_dom::Listener;

use super::body::body_node::BodyNode;

#[derive(Debug, Clone, Default)]
pub struct A {
    attrs: Vec<(&'static str, &'static str)>,
    text: Cow<'static, str>,
    href: Cow<'static, str>,
    #[cfg(feature = "with_yew")]
    listeners: Vec<Rc<dyn Listener>>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for A {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vnode = yew::virtual_dom::VTag::new("a");
        vnode.add_attribute("href", self.href);
        for (a, b) in self.attrs {
            vnode.add_attribute(a, b.to_string())
        }
        vnode.add_child(yew::virtual_dom::VText::new(String::from(self.text)).into());
        vnode.into()
    }
}

impl A {
    pub fn new<S>(href: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self {
            href: href.into(),
            ..Default::default()
        }
    }
    pub fn target(mut self, target: &'static str) -> Self {
        self.attrs.push(("target", target));
        self
    }
    pub fn text<S>(mut self, text: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.text = text.into();
        self
    }
    add_single_attribute!('static);
    to_html!();
    #[cfg(feature = "with_yew")]
    pub fn listener(mut self, listener: Rc<dyn Listener>) -> Self {
        self.listeners.push(listener);
        self
    }
}

impl Display for A {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<a")?;
        for attr in &self.attrs {
            f.write_str(" ")?;
            attr.0.fmt(f)?;
            f.write_str("=\"")?;
            attr.1.fmt(f)?;
            f.write_str("\"")?;
        }
        f.write_str("href=\"")?;
        self.href.fmt(f)?;
        f.write_str("\"")?;
        f.write_str(">")?;
        self.text.fmt(f)?;
        f.write_str("</a>")
    }
}
into_grouping_union!(A, BodyNode);
