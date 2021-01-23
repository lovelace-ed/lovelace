use std::{borrow::Cow, collections::HashMap, fmt::Display};

#[cfg(feature = "with_yew")]
use std::rc::Rc;
#[cfg(feature = "with_yew")]
use yew::virtual_dom::Listener;

use crate::{impl_of_heading_new_fn, into_grouping_union, tags::body::body_node::BodyNode};

#[cfg(feature = "with_yew")]
use crate::heading_of_vnode;

#[derive(Default, Debug, Clone)]
pub struct Text(
    Cow<'static, str>,
    HashMap<&'static str, Cow<'static, str>>,
    #[cfg(feature = "with_yew")] Vec<Rc<dyn Listener>>,
);

impl_of_heading_new_fn!(Text);

#[cfg(feature = "with_yew")]
heading_of_vnode!(Text);

into_grouping_union!(Text, BodyNode);

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
