//! A small set of data types for producing HTML code.
#![allow(clippy::useless_format)]

use std::{fmt::Display, io::Cursor};

use rocket::{response::Responder, Response};

macro_rules! heading_display {
    ($name:ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("<")?;
                f.write_str(stringify!($name))?;
                f.write_str(">")?;
                self.0.fmt(f)?;
                f.write_str("</")?;
                f.write_str(stringify!($name))?;
                f.write_str(">")
            }
        }
    };
}

#[derive(Clone, Debug)]
pub struct Html {
    status: u16,
    reason: Option<&'static str>,
    head: Head,
    body: Body,
}

impl Default for Html {
    fn default() -> Self {
        Self {
            status: 200,
            reason: None,
            head: Head::default(),
            body: Body::default(),
        }
    }
}

impl Display for Html {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<!DOCTYPE html>")?;
        f.write_str("<html>")?;
        self.head.fmt(f)?;
        self.body.fmt(f)?;
        f.write_str("</html>")?;
        Ok(())
    }
}

impl<'r> Responder<'r> for Html {
    fn respond_to(self, _: &rocket::Request) -> rocket::response::Result<'r> {
        Response::build()
            .raw_status(self.status, self.reason.unwrap_or(""))
            .raw_header("Content-Type", "text/html")
            .sized_body(Cursor::new(self.to_string()))
            .ok()
    }
}

impl Html {
    pub fn head(mut self, head: Head) -> Self {
        self.head = head;
        self
    }
    pub fn body(mut self, body: Body) -> Self {
        self.body = body;
        self
    }
    pub fn status(mut self, code: u16) -> Self {
        self.status = code;
        self
    }
    pub fn status_reason(mut self, reason: &'static str) -> Self {
        self.reason = Some(reason);
        self
    }
}

#[derive(Default, Debug, Clone)]
pub struct Head {
    children: Vec<HeadNode>,
}

impl Head {
    pub fn children<C>(mut self, children: Vec<C>) -> Self
    where
        C: Into<HeadNode>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<HeadNode>,
    {
        self.children.push(child.into());
        self
    }
}

impl Display for Head {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<head>")?;
        for child in &self.children {
            child.fmt(f)?;
        }
        f.write_str("</head>")
    }
}

#[derive(Debug, Clone)]
pub enum HeadNode {
    Title(Title),
    Meta(Meta),
}

#[derive(Default, Debug, Clone)]
pub struct Meta {
    attrs: Vec<(String, String)>,
}

impl Meta {
    #[inline(always)]
    pub fn attribute(mut self, k: String, v: String) -> Self {
        self.attrs.push((k, v));
        self
    }
}

impl Display for Meta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<meta")?;
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

#[derive(Debug, Clone)]
pub struct Stylesheet {
    href: String,
}

impl Display for Stylesheet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<link rel=\"stylesheet\" href=\"")?;
        f.write_str(&self.href)?;
        f.write_str("\"/>")
    }
}

impl Stylesheet {
    pub fn new(href: String) -> Self {
        Self { href }
    }
}

#[derive(Debug, Clone)]
pub struct Title(pub String);

heading_display!(Title);

#[derive(Default, Debug, Clone)]
pub struct Body {
    children: Vec<BodyNode>,
}

impl Body {
    pub fn children<C>(mut self, children: Vec<C>) -> Self
    where
        C: Into<BodyNode>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<BodyNode>,
    {
        self.children.push(child.into());
        self
    }
}

impl Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<body>")?;
        for node in &self.children {
            node.fmt(f)?;
        }
        f.write_str("</body>")
    }
}

macro_rules! enum_display {
    ($on:ident, $($variant:ident),*) => {
        impl std::fmt::Display for $on {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$variant(x) => <$variant as std::fmt::Display>::fmt(x, f)),*
                }
            }
        }
    };
}

enum_display!(HeadNode, Title, Meta);

#[derive(Clone, Debug)]
pub enum BodyNode {
    H1(H1),
    H2(H2),
    H3(H3),
    H4(H4),
    H5(H5),
    H6(H6),
    P(P),
    Text(Text),
    Form(Form),
    Br(Br),
    Div(Div),
    A(A),
}

#[derive(Debug, Clone, Default)]
pub struct A {
    attrs: Vec<(String, String)>,
    text: String,
}

impl A {
    pub fn new(href: String) -> Self {
        Self {
            attrs: vec![(format!("href"), href)],
            ..Default::default()
        }
    }
    pub fn target(mut self, target: String) -> Self {
        self.attrs.push((format!("target"), target));
        self
    }
    pub fn text(mut self, text: String) -> Self {
        self.text = text;
        self
    }
}

impl Display for A {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<a")?;
        for attr in &self.attrs {
            f.write_str(" ")?;
            attr.0.fmt(f)?;
            f.write_str("=\"")?;
            attr.1.fmt(f)?;
            f.write_str("\"")?;
        }
        f.write_str(">")?;
        self.text.fmt(f)?;
        f.write_str("</a>")
    }
}

#[derive(Debug, Clone, Default)]
pub struct Div {
    children: Vec<BodyNode>,
    attrs: Vec<(String, String)>,
}

impl Div {
    pub fn children<C, D>(mut self, children: C) -> Self
    where
        C: IntoIterator<Item = D>,
        D: Into<BodyNode>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<BodyNode>,
    {
        self.children.push(child.into());
        self
    }
    pub fn map<F>(mut self, mapping: F) -> Self
    where
        F: Fn(Self) -> Self,
    {
        self = mapping(self);
        self
    }
    #[inline(always)]
    pub fn attribute(mut self, k: String, v: String) -> Self {
        self.attrs.push((k, v));
        self
    }
}

impl Display for Div {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<div")?;
        for attr in &self.attrs {
            f.write_str(" ")?;
            attr.0.fmt(f)?;
            f.write_str("=\"")?;
            attr.1.fmt(f)?;
            f.write_str("\"")?;
        }
        f.write_str("/>")?;
        for node in &self.children {
            node.fmt(f)?;
        }
        f.write_str("</div>")
    }
}

#[derive(Debug, Clone)]
pub struct Br;

impl Display for Br {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<br/>")
    }
}

macro_rules! into_grouping_union {
    ($name:ident, $enum_name:ident) => {
        impl From<$name> for $enum_name {
            fn from(t: $name) -> $enum_name {
                $enum_name::$name(t)
            }
        }
    };
}

into_grouping_union!(Div, BodyNode);

into_grouping_union!(Br, BodyNode);
into_grouping_union!(Br, FormNode);

into_grouping_union!(Meta, HeadNode);
into_grouping_union!(Title, HeadNode);

into_grouping_union!(A, BodyNode);

enum_display!(BodyNode, H1, H2, H3, H4, H5, H6, P, Br, Text, Form, Div, A);

#[derive(Default, Debug, Clone)]
pub struct H1(pub String);

into_grouping_union!(H1, BodyNode);

heading_display!(H1);

#[derive(Default, Debug, Clone)]
pub struct H2(pub String);

into_grouping_union!(H2, BodyNode);

heading_display!(H2);

#[derive(Default, Debug, Clone)]
pub struct H3(pub String);

into_grouping_union!(H3, BodyNode);

heading_display!(H3);

#[derive(Default, Debug, Clone)]
pub struct H4(pub String);

into_grouping_union!(H4, BodyNode);

heading_display!(H4);

#[derive(Default, Debug, Clone)]
pub struct H5(pub String);

into_grouping_union!(H5, BodyNode);

heading_display!(H5);

#[derive(Default, Debug, Clone)]
pub struct H6(pub String);

into_grouping_union!(H6, BodyNode);

heading_display!(H6);

#[derive(Default, Debug, Clone)]
pub struct P {
    children: Vec<BodyNode>,
}

into_grouping_union!(P, BodyNode);

impl Display for P {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<p>")?;
        for child in &self.children {
            child.fmt(f)?;
        }
        f.write_str("</p>")
    }
}

impl P {
    pub fn children<C>(mut self, children: Vec<C>) -> Self
    where
        C: Into<BodyNode>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<BodyNode>,
    {
        self.children.push(child.into());
        self
    }
    pub fn with_text(text: String) -> Self {
        P::default().child(BodyNode::Text(Text(text)))
    }
    pub fn text(self, text: String) -> Self {
        self.child(BodyNode::Text(Text(text)))
    }
}

#[derive(Default, Debug, Clone)]
pub struct Text(pub String);

into_grouping_union!(Text, BodyNode);

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Form {
    children: Vec<FormNode>,
    attrs: Vec<(String, String)>,
}

impl Form {
    #[inline(always)]
    pub fn children<C>(mut self, children: Vec<C>) -> Self
    where
        C: Into<FormNode>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
    #[inline(always)]
    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<FormNode>,
    {
        self.children.push(child.into());
        self
    }
    #[inline(always)]
    pub fn attribute(mut self, k: String, v: String) -> Self {
        self.attrs.push((k, v));
        self
    }
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

#[derive(Debug, Clone)]
pub enum FormNode {
    Input(Input),
    Label(Label),
    Br(Br),
}

enum_display!(FormNode, Input, Label, Br);

#[derive(Debug, Clone, Default)]
pub struct Input {
    attrs: Vec<(String, String)>,
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

into_grouping_union!(Input, FormNode);

impl Input {
    #[inline(always)]
    pub fn attribute(mut self, k: String, v: String) -> Self {
        self.attrs.push((k, v));
        self
    }
}

#[derive(Debug, Clone)]
pub struct Label(pub String);

heading_display!(Label);

into_grouping_union!(Label, FormNode);
