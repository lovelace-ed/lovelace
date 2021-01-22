use std::{borrow::Cow, collections::HashMap};

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
