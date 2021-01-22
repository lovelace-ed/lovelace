#[cfg(feature = "with_yew")]
use std::rc::Rc;
use std::{borrow::Cow, collections::HashMap};
#[cfg(feature = "with_yew")]
use yew::virtual_dom::Listener;

#[cfg(feature = "with_yew")]
use crate::heading_of_vnode;
use crate::{heading_display, impl_of_heading_new_fn, into_grouping_union};

use super::body::body_node::BodyNode;

#[derive(Default, Debug, Clone)]
pub struct H1(
    Cow<'static, str>,
    #[cfg(feature = "with_yew")] Vec<Rc<dyn Listener>>,
    HashMap<&'static str, String>,
);

impl_of_heading_new_fn!(H1);

into_grouping_union!(H1, BodyNode);

heading_display!(H1);

#[derive(Default, Debug, Clone)]
pub struct H2(
    Cow<'static, str>,
    #[cfg(feature = "with_yew")] Vec<Rc<dyn Listener>>,
    HashMap<&'static str, String>,
);

impl_of_heading_new_fn!(H2);

into_grouping_union!(H2, BodyNode);

heading_display!(H2);

#[derive(Default, Debug, Clone)]
pub struct H3(
    Cow<'static, str>,
    #[cfg(feature = "with_yew")] Vec<Rc<dyn Listener>>,
    HashMap<&'static str, String>,
);

impl_of_heading_new_fn!(H3);

into_grouping_union!(H3, BodyNode);

heading_display!(H3);

#[derive(Default, Debug, Clone)]
pub struct H4(
    Cow<'static, str>,
    #[cfg(feature = "with_yew")] Vec<Rc<dyn Listener>>,
    HashMap<&'static str, String>,
);

impl_of_heading_new_fn!(H4);

into_grouping_union!(H4, BodyNode);

heading_display!(H4);

#[derive(Default, Debug, Clone)]
pub struct H5(
    Cow<'static, str>,
    #[cfg(feature = "with_yew")] Vec<Rc<dyn Listener>>,
    HashMap<&'static str, String>,
);

impl_of_heading_new_fn!(H5);

into_grouping_union!(H5, BodyNode);

heading_display!(H5);

#[derive(Default, Debug, Clone)]
pub struct H6(
    Cow<'static, str>,
    #[cfg(feature = "with_yew")] Vec<Rc<dyn Listener>>,
    HashMap<&'static str, String>,
);

impl_of_heading_new_fn!(H6);

into_grouping_union!(H6, BodyNode);

heading_display!(H6);

#[cfg(feature = "with_yew")]
heading_of_vnode!(H1);
#[cfg(feature = "with_yew")]
heading_of_vnode!(H2);
#[cfg(feature = "with_yew")]
heading_of_vnode!(H3);
#[cfg(feature = "with_yew")]
heading_of_vnode!(H4);
#[cfg(feature = "with_yew")]
heading_of_vnode!(H5);
#[cfg(feature = "with_yew")]
heading_of_vnode!(H6);

#[test]
fn test_headings() {
    use crate::prelude::*;
    let document = Html::default()
        .head(Head::default().child(Title::new("Some title")))
        .body(
            Body::default()
                .child(H6::new("Some heading"))
                .child(H6::new("Some other heading"))
                .child(H5::new("Some other other heading"))
                .child(
                    H4::new("Some other other other heading").attribute("class", "heading-class"),
                ),
        )
        .to_string();
    let document = scraper::Html::parse_document(&document);
    let h6_selector = scraper::Selector::parse("h6").unwrap();
    let h5_selector = scraper::Selector::parse("h5").unwrap();
    let h4_selector = scraper::Selector::parse("h4").unwrap();
    assert_eq!(document.select(&h6_selector).collect::<Vec<_>>().len(), 2);
    assert_eq!(document.select(&h5_selector).collect::<Vec<_>>().len(), 1);
    assert_eq!(
        document
            .select(&h6_selector)
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap(),
        "Some heading"
    );
    assert_eq!(
        document
            .select(&h5_selector)
            .next()
            .unwrap()
            .text()
            .next()
            .unwrap(),
        "Some other other heading"
    );
    let h4 = document.select(&h4_selector).next().unwrap();
    assert_eq!(h4.text().next().unwrap(), "Some other other other heading");
    assert_eq!(h4.value().attr("class").unwrap(), "heading-class");
}
