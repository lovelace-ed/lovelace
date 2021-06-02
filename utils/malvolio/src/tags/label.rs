/*
This source code file is distributed subject to the terms of the Mozilla Public License v2.0.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
use std::{borrow::Cow, collections::HashMap};

#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
use std::rc::Rc;
#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
use yew::virtual_dom::Listener;

#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
use crate::heading_of_vnode;
use crate::{heading_display, impl_of_heading_new_fn, into_grouping_union};

use super::body::body_node::BodyNode;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "pub_fields", derive(FieldsAccessibleVariant))]
/// A label for a form.
///
/// See the [MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/label)
/// for further information.
pub struct Label {
    text: Cow<'static, str>,
    attrs: HashMap<&'static str, Cow<'static, str>>,
    #[cfg(feature = "with_yew")]
    #[cfg(not(tarpaulin))]
    listeners: Vec<Rc<dyn Listener>>,
}

#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
heading_of_vnode!(Label);

impl_of_heading_new_fn!(Label, label);

heading_display!(Label);

into_grouping_union!(Label, BodyNode);

#[cfg(test)]
mod test {
    use crate::prelude::*;
    #[test]
    fn test_p() {
        let document = Label::new("Label text").to_string();
        let document = scraper::Html::parse_document(&document);
        let label = scraper::Selector::parse("label").unwrap();
        let label = document.select(&label).next().unwrap();
        assert_eq!(
            label
                .children()
                .next()
                .unwrap()
                .value()
                .as_text()
                .unwrap()
                .to_string()
                .as_str(),
            "Label text"
        );
    }
}
