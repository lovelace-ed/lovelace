use std::{borrow::Cow, collections::HashMap, fmt::Display};

#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;
use crate::{
    attributes::IntoAttribute, into_attribute_for_grouping_enum, into_grouping_union, to_html,
    utility_enum,
};

use super::body::body_node::BodyNode;

#[derive(Debug, Clone, Default)]
pub struct Form {
    children: Vec<BodyNode>,
    attrs: HashMap<&'static str, Cow<'static, str>>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Form {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("form");
        vtag.add_children(self.children.into_iter().map(IntoVNode::into));
        for (a, b) in self.attrs {
            vtag.add_attribute(a, &b.to_string())
        }
        vtag.into()
    }
}

impl Form {
    #[inline(always)]
    pub fn children<C>(mut self, children: Vec<C>) -> Self
    where
        C: Into<BodyNode>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
    #[inline(always)]
    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<BodyNode>,
    {
        self.children.push(child.into());
        self
    }
    pub fn attribute<A>(mut self, attr: A) -> Self
    where
        A: Into<FormAttr>,
    {
        let res = attr.into().into_attribute();
        self.attrs.insert(res.0, res.1);
        self
    }
    to_html!();
}

impl Display for Form {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<form ")?;
        for attr in &self.attrs {
            f.write_str(" ")?;
            attr.0.fmt(f)?;
            f.write_str("=\"")?;
            attr.1.fmt(f)?;
            f.write_str("\"")?;
        }
        f.write_str(">")?;
        for node in &self.children {
            node.fmt(f)?;
        }
        f.write_str("</form>")
    }
}

into_grouping_union!(Form, BodyNode);

utility_enum!(
    pub enum FormAttr {
        Method(Method),
        Action(Action),
    }
);

pub enum Method {
    Post,
    Get,
}

into_attribute_for_grouping_enum!(FormAttr, Method, Action);

impl IntoAttribute for Method {
    fn into_attribute(self) -> (&'static str, Cow<'static, str>) {
        (
            "method",
            match self {
                Method::Post => "post",
                Method::Get => "get",
            }
            .into(),
        )
    }
}

into_grouping_union!(Method, FormAttr);

pub struct Action(Cow<'static, str>);

impl Action {
    pub fn new<S>(input: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(input.into())
    }
}

impl IntoAttribute for Action {
    fn into_attribute(self) -> (&'static str, Cow<'static, str>) {
        ("action", self.0)
    }
}

into_grouping_union!(Action, FormAttr);

#[cfg(test)]
mod form {
    use crate::{
        prelude::*,
        tags::input::{Name, Type},
    };

    use super::{Action, Method};
    #[test]
    fn test_form_tag() {
        let document = Form::default()
            .attribute(Method::Post)
            .attribute(Action::new("/"))
            .to_string();
        let document = scraper::Html::parse_document(&document);
        let form = scraper::Selector::parse("form").unwrap();
        let form = document.select(&form).next().unwrap().value();
        assert_eq!(form.attr("method"), Some("post"));
        assert_eq!(form.attr("action"), Some("/"));
    }
    #[test]
    fn test_form_with_children() {
        let document = Form::default()
            .child(
                Input::default()
                    .attribute(Type::Text)
                    .attribute(Name::new("input1")),
            )
            .child(Input::default().attribute(Type::Submit))
            .to_string();
        let document = scraper::Html::parse_document(&document);
        let input = scraper::Selector::parse("input").unwrap();
        let inputs = document.select(&input).collect::<Vec<_>>();
        assert_eq!(inputs.len(), 2);
        let input1 = inputs[0].value();
        assert_eq!(input1.attr("type"), Some("text"));
        assert_eq!(input1.attr("name"), Some("input1"));
        let input2 = inputs[1].value();
        assert_eq!(input2.attr("type"), Some("submit"))
    }
}
