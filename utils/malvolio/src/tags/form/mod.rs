use std::{borrow::Cow, collections::HashMap, fmt::Display};

#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;
use crate::{
    attributes::IntoAttribute, into_attribute_for_grouping_enum, into_grouping_union, to_html,
    utility_enum,
};

use crate::tags::body::body_node::BodyNode;

#[derive(Debug, Clone, Derivative)]
#[derivative(Default(new = "true"))]
/// A HTML form. You can create a form with `Form::new()` or `Form::default()` (they are identical)
/// and then use any of the provided methods to manipulate it (for example adding child elements or
/// attributes).
///
/// ```
/// # use malvolio::prelude::*;
/// malvolio::prelude::Form::new()
/// .attribute(Method::Post)
/// .child(
///     Input::default()
///         .attribute(Type::Text)
///         .attribute(Placeholder::new("Username"))
///         .attribute(Name::new("username")),
/// )
/// .child(Br)
/// .child(
///     Input::new()
///         .attribute(Type::Email)
///         .attribute(Placeholder::new("Email"))
///         .attribute(Name::new("email")),
/// )
/// .child(Br)
/// .child(
///     Input::new()
///         .attribute(Type::Password)
///         .attribute(Placeholder::new("Password"))
///         .attribute(Name::new("password")),
/// )
/// .child(Br)
/// .child(
///     Input::new()
///         .attribute(Type::Password)
///         .attribute(Placeholder::new("Password confirmation"))
///         .attribute(Name::new("password_confirmation")),
/// )
/// .child(Br)
/// .child(
///     Input::new()
///         .attribute(Type::Submit)
///         .attribute(Value::new("Login!")),
/// );
/// ```
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
    /// Add a number of children to a form. This method accepts a single argument which must
    /// implement `IntoIterator` (so for example a `Vec`) where the item of the iterator implements
    /// `Into<BodyNode>` (any of the types in this crate which you would expect to go in the body of
    /// an HTML document should implement this).
    ///
    /// ```
    /// # use malvolio::prelude::*;
    /// Form::new().children(vec![1, 2, 3].into_iter().map(|item| {
    ///     Label::new(format!("Item number: {}", item))
    /// }));
    /// ```
    pub fn children<I, C>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<BodyNode>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
    /// Add a single child to a form. This method accepts a single item implementing
    /// `Into<BodyNode>`.
    /// ```
    /// # use malvolio::prelude::*;
    /// Form::new()
    ///     .child(Label::new("Some input"))
    ///     .child(Input::new().attribute(Name::new("some-input")));
    /// ```
    #[inline(always)]
    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<BodyNode>,
    {
        self.children.push(child.into());
        self
    }
    /// Add an attribute to the current form. This accepts any item implementing `Into<FormAttr>`
    /// (which is all the members of the `FormAttr` enum).
    ///
    /// ```
    /// # use malvolio::prelude::*;
    /// Form::new()
    ///     .attribute(Method::Post)
    ///     .attribute(Action::new("/"));
    /// ```
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

/// The "method" attribute for a form. See the
/// [MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/form#attr-method) for
/// further details.
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

/// The "action" attribute for a form. See the
/// [MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/form#attr-action) for
/// further details.
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
        let document = Form::new()
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
        let document = Form::new()
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
