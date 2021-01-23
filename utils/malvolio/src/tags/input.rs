#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;
use crate::{
    attributes::IntoAttribute,
    into_attribute_for_grouping_enum, into_grouping_union,
    prelude::{Class, Id},
    to_html, utility_enum,
};
#[cfg(feature = "with_yew")]
use std::rc::Rc;
use std::{borrow::Cow, collections::HashMap, fmt::Display};
#[cfg(feature = "with_yew")]
use yew::virtual_dom::Listener;

use super::body::body_node::BodyNode;

#[derive(Debug, Clone, Derivative)]
#[derivative(Default(new = "true"))]
pub struct Input {
    attrs: HashMap<&'static str, Cow<'static, str>>,
    #[cfg(feature = "with_yew")]
    listeners: Vec<Rc<dyn Listener>>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Input {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("input");
        self.attrs
            .clone()
            .iter()
            .find(|item| item.0 == &"type")
            .map(|(_, res)| vtag.set_kind(&res.to_string()));
        for (a, b) in self.attrs {
            vtag.add_attribute(a, &b)
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
    pub fn attribute<C>(mut self, c: C) -> Self
    where
        C: Into<InputAttr>,
    {
        let (a, b) = c.into().into_attribute();
        self.attrs.insert(a, b);
        self
    }
    #[cfg(feature = "with_yew")]
    pub fn listener(mut self, listener: Rc<dyn Listener>) -> Self {
        self.listeners.push(listener);
        self
    }
    #[cfg(feature = "with_yew")]
    pub fn listeners(mut self, listeners: Vec<Rc<dyn Listener>>) -> Self {
        self.listeners.extend(listeners);
        self
    }
    to_html!();
}

utility_enum!(
    pub enum InputAttr {
        Type(Type),
        Name(Name),
        Placeholder(Placeholder),
        Id(Id),
        Class(Class),
        Value(Value),
    }
);

into_attribute_for_grouping_enum!(InputAttr, Type, Name, Placeholder, Id, Class, Value);

into_grouping_union!(Id, InputAttr);
into_grouping_union!(Class, InputAttr);

pub enum Type {
    Text,
    Email,
    Password,
    Textarea,
    Submit,
    Hidden,
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
            }
            .into(),
        )
    }
}

into_grouping_union!(Type, InputAttr);

pub struct Name(Cow<'static, str>);

impl IntoAttribute for Name {
    fn into_attribute(self) -> (&'static str, Cow<'static, str>) {
        ("name", self.0)
    }
}

impl Name {
    pub fn new<S>(s: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(s.into())
    }
}

into_grouping_union!(Name, InputAttr);

pub struct Placeholder(Cow<'static, str>);

impl IntoAttribute for Placeholder {
    fn into_attribute(self) -> (&'static str, Cow<'static, str>) {
        ("placeholder", self.0)
    }
}

impl Placeholder {
    pub fn new<S>(s: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(s.into())
    }
}

into_grouping_union!(Placeholder, InputAttr);

pub struct Value(Cow<'static, str>);

impl Value {
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

into_grouping_union!(Value, InputAttr);

#[cfg(test)]
#[cfg(feature = "with_yew")]
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
