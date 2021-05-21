/*
This source code file is distributed subject to the terms of the Mozilla Public License v2.0.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
use crate::into_vnode::IntoVNode;
use crate::{
    attributes::IntoAttribute,
    into_attribute_for_grouping_enum, into_grouping_union,
    prelude::{Class, Id, Style},
    to_html, utility_enum,
};
#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
use std::rc::Rc;
use std::{borrow::Cow, collections::HashMap, fmt::Display};
#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
use yew::virtual_dom::Listener;

use super::body::body_node::BodyNode;

#[derive(Debug, Clone, Derivative)]
#[derivative(Default(new = "true"))]
/// A form input.
///
/// See the [MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input)
/// for more info.
pub struct Input {
    attrs: HashMap<&'static str, Cow<'static, str>>,
    #[cfg(feature = "with_yew")]
    #[cfg(not(tarpaulin))]
    listeners: Vec<Rc<dyn Listener>>,
}

#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
impl IntoVNode for Input {
    fn into_vnode(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("input");
        self.attrs
            .clone()
            .iter()
            .find(|item| item.0 == &"type")
            .map(|(_, res)| vtag.set_kind(res.to_string()));
        for (a, b) in self.attrs {
            vtag.add_attribute(a, b)
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
    /// Attach a new attribute to this type.
    pub fn attribute<C>(mut self, c: C) -> Self
    where
        C: Into<InputAttr>,
    {
        let (a, b) = c.into().into_attribute();
        self.attrs.insert(a, b);
        self
    }
    /// Read an attribute that has been set
    pub fn read_attribute(&self, attribute: &'static str) -> Option<&Cow<'static, str>> {
        self.attrs.get(attribute)
    }
    #[cfg(feature = "with_yew")]
    #[cfg(not(tarpaulin))]
    /// Attaches a listener to the input item. Note that this is only available if you have enabled
    /// the `with_yew` feature.
    pub fn listener(mut self, listener: Rc<dyn Listener>) -> Self {
        self.listeners.push(listener);
        self
    }
    /// Attaches multiple listeners to the input item. Note that this is only available if you have
    /// enabled the `with_yew` feature.
    #[cfg(feature = "with_yew")]
    #[cfg(not(tarpaulin))]
    pub fn listeners<I, T>(mut self, listeners: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Rc<dyn Listener>>,
    {
        self.listeners.extend(listeners.into_iter().map(Into::into));
        self
    }
    /// Apply a function to this tag.
    pub fn map<F>(self, mapping: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        mapping(self)
    }
    to_html!();
}

utility_enum!(
    /// The possible attributes which can be attached to an input item.
    #[allow(missing_docs)]
    pub enum InputAttr {
        Type(Type),
        Name(Name),
        Placeholder(Placeholder),
        Id(Id),
        Class(Class),
        Value(Value),
        Style(Style),
    }
);

into_attribute_for_grouping_enum!(InputAttr, Type, Name, Placeholder, Id, Class, Value, Style);

into_grouping_union!(Id, InputAttr);
into_grouping_union!(Class, InputAttr);
into_grouping_union!(Value, InputAttr);
into_grouping_union!(Style, InputAttr);
into_grouping_union!(Name, InputAttr);
into_grouping_union!(Type, InputAttr);
into_grouping_union!(Placeholder, InputAttr);

/// The `type` attribute for an input.
///
/// See the [MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input#attr-type)
/// for more info.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum Type {
    Text,
    Email,
    Password,
    Textarea,
    Submit,
    Hidden,
    DateTimeLocal,
    Checkbox,
}

impl IntoAttribute for Type {
    fn into_attribute(self) -> (&'static str, Cow<'static, str>) {
        (
            "type",
            match self {
                Type::Text => "text",
                Type::Email => "email",
                Type::Password => "password",
                Type::Submit => "submit",
                Type::Textarea => "textarea",
                Type::Hidden => "hidden",
                Type::DateTimeLocal => "datetime-local",
                Type::Checkbox => "checkbox",
            }
            .into(),
        )
    }
}

/// The `name` attribute for an input.
///
/// See the [MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input#attr-name)
/// for more info.
#[derive(Debug, Clone)]

pub struct Name(Cow<'static, str>);

impl IntoAttribute for Name {
    fn into_attribute(self) -> (&'static str, Cow<'static, str>) {
        ("name", self.0)
    }
}

impl Name {
    /// Create a new instance of this attribute with the specified value.
    pub fn new<S>(s: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(s.into())
    }
}

/// The "placeholder" attribute for an input field.
///
/// See the [MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input#attr-placeholder)
/// for more info.
#[derive(Debug, Clone)]
pub struct Placeholder(Cow<'static, str>);

impl IntoAttribute for Placeholder {
    fn into_attribute(self) -> (&'static str, Cow<'static, str>) {
        ("placeholder", self.0)
    }
}

impl Placeholder {
    /// Create a new instance of this attribute with the specified value.
    pub fn new<S>(s: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(s.into())
    }
}

/// The "value" attribute for an input field.
///
/// See the [MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input#attr-value)
/// for more info.
#[derive(Debug, Clone)]
pub struct Value(Cow<'static, str>);

impl Value {
    /// Create a new instance of this attribute with the specified value.
    pub fn new<S>(s: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(s.into())
    }
}

impl IntoAttribute for Value {
    fn into_attribute(self) -> (&'static str, Cow<'static, str>) {
        ("value", self.0)
    }
}

#[cfg(test)]
#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
mod test_yew {
    use crate::component_named_app_with_html;
    use crate::prelude::*;
    use wasm_bindgen_test::*;
    use yew::prelude::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_input_in_browser() {
        component_named_app_with_html!(Input::default()
            .attribute(Id::new("some-id"))
            .attribute(Placeholder::new("some-placeholder"))
            .attribute(Value::new("some-value"))
            .to_html());
        let document = web_sys::window().unwrap().document().unwrap();
        let root = document
            .create_element("div")
            .expect("failed to create element");
        yew::App::<App>::new().mount(root.clone());
        let input = root
            .get_elements_by_tag_name("input")
            .named_item("some-id")
            .unwrap();
        assert_eq!(
            input
                .attributes()
                .get_named_item("value")
                .expect("failed to get placeholder")
                .value(),
            "some-value"
        );
        assert_eq!(
            input
                .attributes()
                .get_named_item("id")
                .expect("failed to get placeholder")
                .value(),
            "some-id"
        );
        assert_eq!(
            input
                .attributes()
                .get_named_item("placeholder")
                .expect("failed to get placeholder")
                .value(),
            "some-placeholder"
        );
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    #[test]
    fn test_input() {
        let document = Input::default()
            .attribute(Id::new("some-id"))
            .attribute(Placeholder::new("some-placeholder"))
            .attribute(Value::new("some-value"))
            .to_string();
        let document = scraper::Html::parse_document(&document);
        let input = scraper::Selector::parse("input").unwrap();
        let input = document.select(&input).next().unwrap().value();
        assert_eq!(input.attr("id"), Some("some-id"));
        assert_eq!(input.attr("placeholder"), Some("some-placeholder"));
        assert_eq!(input.attr("value"), Some("some-value"));
    }
}
