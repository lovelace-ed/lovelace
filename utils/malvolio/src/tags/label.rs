use std::{borrow::Cow, collections::HashMap};

#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;
#[cfg(feature = "with_yew")]
use std::rc::Rc;
#[cfg(feature = "with_yew")]
use yew::virtual_dom::Listener;

#[cfg(feature = "with_yew")]
use crate::heading_of_vnode;
use crate::{heading_display, impl_of_heading_new_fn, into_grouping_union};

use super::body::body_node::BodyNode;

#[derive(Debug, Clone)]
pub struct Label(
    Cow<'static, str>,
    #[cfg(feature = "with_yew")] Vec<Rc<dyn Listener>>,
    HashMap<&'static str, String>,
);

#[cfg(feature = "with_yew")]
heading_of_vnode!(Label);

impl_of_heading_new_fn!(Label);

heading_display!(Label);

into_grouping_union!(Label, BodyNode);
