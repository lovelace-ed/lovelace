/*
This source code file is distributed subject to the terms of the Mozilla Public License v2.0.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
use std::{borrow::Cow, collections::HashMap, fmt::Display};

#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
use std::rc::Rc;
#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
use yew::virtual_dom::Listener;

use crate::{impl_of_heading_new_fn, into_grouping_union, tags::body::body_node::BodyNode};

#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
use crate::heading_of_vnode;

#[derive(Default, Debug, Clone)]
/// A text node.
pub struct Text(
    Cow<'static, str>,
    HashMap<&'static str, Cow<'static, str>>,
    #[cfg(feature = "with_yew")]
    #[cfg(not(tarpaulin))]
    Vec<Rc<dyn Listener>>,
);

impl_of_heading_new_fn!(Text);

#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
heading_of_vnode!(Text);

into_grouping_union!(Text, BodyNode);

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
