/*
This source code file is distributed subject to the terms of the Mozilla Public License v2.0.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
use std::rc::Rc;
use std::{borrow::Cow, collections::HashMap};
#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
use yew::virtual_dom::Listener;

#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
use crate::heading_of_vnode;
use crate::{
    heading_display, impl_of_heading_new_fn, into_attribute_for_grouping_enum, into_grouping_union,
    prelude::{Class, Id},
    utility_enum,
};

use super::body::body_node::BodyNode;

#[derive(Default, Debug, Clone)]
/// The <h1> tag.
pub struct H1(
    Cow<'static, str>,
    HashMap<&'static str, Cow<'static, str>>,
    #[cfg(feature = "with_yew")]
    #[cfg(not(tarpaulin))]
    Vec<Rc<dyn Listener>>,
);

impl_of_heading_new_fn!(H1);

into_grouping_union!(H1, BodyNode);

heading_display!(H1);

#[derive(Default, Debug, Clone)]
/// The <h2> tag.
pub struct H2(
    Cow<'static, str>,
    HashMap<&'static str, Cow<'static, str>>,
    #[cfg(feature = "with_yew")]
    #[cfg(not(tarpaulin))]
    Vec<Rc<dyn Listener>>,
);

impl_of_heading_new_fn!(H2);

into_grouping_union!(H2, BodyNode);

heading_display!(H2);

#[derive(Default, Debug, Clone)]
/// The <h3> tag.
pub struct H3(
    Cow<'static, str>,
    HashMap<&'static str, Cow<'static, str>>,
    #[cfg(feature = "with_yew")]
    #[cfg(not(tarpaulin))]
    Vec<Rc<dyn Listener>>,
);

impl_of_heading_new_fn!(H3);

into_grouping_union!(H3, BodyNode);

heading_display!(H3);

#[derive(Default, Debug, Clone)]
/// The <h4> tag.
pub struct H4(
    Cow<'static, str>,
    HashMap<&'static str, Cow<'static, str>>,
    #[cfg(feature = "with_yew")]
    #[cfg(not(tarpaulin))]
    Vec<Rc<dyn Listener>>,
);

impl_of_heading_new_fn!(H4);

into_grouping_union!(H4, BodyNode);

heading_display!(H4);

#[derive(Default, Debug, Clone)]
/// The <h5> tag.
pub struct H5(
    Cow<'static, str>,
    HashMap<&'static str, Cow<'static, str>>,
    #[cfg(feature = "with_yew")]
    #[cfg(not(tarpaulin))]
    Vec<Rc<dyn Listener>>,
);

impl_of_heading_new_fn!(H5);

into_grouping_union!(H5, BodyNode);

heading_display!(H5);

#[derive(Default, Debug, Clone)]
/// The <h6> tag.
pub struct H6(
    Cow<'static, str>,
    HashMap<&'static str, Cow<'static, str>>,
    #[cfg(feature = "with_yew")]
    #[cfg(not(tarpaulin))]
    Vec<Rc<dyn Listener>>,
);

impl_of_heading_new_fn!(H6);

into_grouping_union!(H6, BodyNode);

heading_display!(H6);

#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
heading_of_vnode!(H1);
#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
heading_of_vnode!(H2);
#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
heading_of_vnode!(H3);
#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
heading_of_vnode!(H4);
#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
heading_of_vnode!(H5);
#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
heading_of_vnode!(H6);

utility_enum!(
    /// An attribute for a heading tag.
    #[allow(missing_docs)]
    pub enum HeadingAttr {
        Class(Class),
        Id(Id),
    }
);
into_attribute_for_grouping_enum!(HeadingAttr, Class, Id);

into_grouping_union!(Class, HeadingAttr);

into_grouping_union!(Id, HeadingAttr);

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
                    H4::new("Some other other other heading")
                        .attribute(Class::from("heading-class")),
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
