use std::{borrow::Cow, collections::HashMap};

#[cfg(feature = "with_yew")]
use std::rc::Rc;
#[cfg(feature = "with_yew")]
use yew::virtual_dom::Listener;

#[cfg(feature = "with_yew")]
use crate::heading_of_vnode;
use crate::{heading_display, impl_of_heading_new_fn, into_grouping_union};

use super::head::head_node::HeadNode;

#[derive(Debug, Clone)]
/// The <title> tag.
///
/// See the [MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/title) for more
/// info.
pub struct Title(
    Cow<'static, str>,
    HashMap<&'static str, Cow<'static, str>>,
    #[cfg(feature = "with_yew")] Vec<Rc<dyn Listener>>,
);

#[cfg(feature = "with_yew")]
heading_of_vnode!(Title);

impl_of_heading_new_fn!(Title);

heading_display!(Title);

into_grouping_union!(Title, HeadNode);
