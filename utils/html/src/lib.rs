//! A small set of data types for producing HTML code.
#![allow(clippy::useless_format)]

use std::{borrow::Cow, fmt::Display, io::Cursor};

use rocket::{response::Responder, Response};

macro_rules! heading_display {
    ($name:ident) => {
        impl<'a> std::fmt::Display for $name<'a> {
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

macro_rules! impl_of_heading_new_fn {
    ($name:ident) => {
        impl<'a> $name<'a> {
            pub fn new<S>(from: S) -> Self
            where
                S: Into<std::borrow::Cow<'a, str>>,
            {
                Self(from.into())
            }
        }
    };
}

#[derive(Clone, Debug)]
pub struct Html<'a> {
    status: u16,
    reason: Option<&'static str>,
    head: Head<'a>,
    body: Body<'a>,
}

impl<'a> Default for Html<'a> {
    fn default() -> Self {
        Self {
            status: 200,
            reason: None,
            head: Head::default(),
            body: Body::default(),
        }
    }
}

impl<'a> Display for Html<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<!DOCTYPE html>")?;
        f.write_str("<html>")?;
        self.head.fmt(f)?;
        self.body.fmt(f)?;
        f.write_str("</html>")?;
        Ok(())
    }
}

impl<'r> Responder<'r> for Html<'_> {
    fn respond_to(self, _: &rocket::Request) -> rocket::response::Result<'r> {
        Response::build()
            .raw_status(self.status, self.reason.unwrap_or(""))
            .raw_header("Content-Type", "text/html")
            .sized_body(Cursor::new(self.to_string()))
            .ok()
    }
}

impl<'a> Html<'a> {
    pub fn head(mut self, head: Head<'a>) -> Self {
        self.head = head;
        self
    }
    pub fn body(mut self, body: Body<'a>) -> Self {
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
pub struct Head<'a> {
    children: Vec<HeadNode<'a>>,
}

impl<'a> Head<'a> {
    pub fn children<C>(mut self, children: Vec<C>) -> Self
    where
        C: Into<HeadNode<'a>>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<HeadNode<'a>>,
    {
        self.children.push(child.into());
        self
    }
}

impl<'a> Display for Head<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<head>")?;
        for child in &self.children {
            child.fmt(f)?;
        }
        f.write_str("</head>")
    }
}

#[derive(Debug, Clone)]
pub enum HeadNode<'a> {
    Title(Title<'a>),
    Meta(Meta<'a>),
}

#[derive(Default, Debug, Clone)]
pub struct Meta<'a> {
    attrs: Vec<(&'a str, &'a str)>,
}

impl<'a> Meta<'a> {
    #[inline(always)]
    pub fn attribute(mut self, k: &'a str, v: &'a str) -> Self {
        self.attrs.push((k, v));
        self
    }
}

impl<'a> Display for Meta<'a> {
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
pub struct Title<'a>(Cow<'a, str>);

impl_of_heading_new_fn!(Title);

heading_display!(Title);

#[derive(Default, Debug, Clone)]
pub struct Body<'a> {
    children: Vec<BodyNode<'a>>,
}

impl<'a> Body<'a> {
    pub fn children<C>(mut self, children: Vec<C>) -> Self
    where
        C: Into<BodyNode<'a>>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<BodyNode<'a>>,
    {
        self.children.push(child.into());
        self
    }
}

impl<'a> Display for Body<'a> {
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
        impl std::fmt::Display for $on <'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$variant(x) => <$variant as std::fmt::Display>::fmt(&x.clone(), f)),*
                }
            }
        }
    };
}

enum_display!(HeadNode, Title, Meta);

#[derive(Clone, Debug)]
pub enum BodyNode<'a> {
    H1(H1<'a>),
    H2(H2<'a>),
    H3(H3<'a>),
    H4(H4<'a>),
    H5(H5<'a>),
    H6(H6<'a>),
    P(P<'a>),
    Text(Text<'a>),
    Form(Form<'a>),
    Br(Br),
    Div(Div<'a>),
    A(A<'a>),
}

#[derive(Debug, Clone, Default)]
pub struct A<'a> {
    attrs: Vec<(&'a str, &'a str)>,
    text: Cow<'a, str>,
    href: Cow<'a, str>,
}

impl<'a> A<'a> {
    pub fn new<S>(href: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self {
            href: href.into(),
            ..Default::default()
        }
    }
    pub fn target(mut self, target: &'a str) -> Self {
        self.attrs.push(("target", target));
        self
    }
    pub fn text<S>(mut self, text: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.text = text.into();
        self
    }
}

impl<'a> Display for A<'a> {
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
pub struct Div<'a> {
    children: Vec<BodyNode<'a>>,
    attrs: Vec<(&'a str, &'a str)>,
}

impl<'a> Div<'a> {
    pub fn children<C, D>(mut self, children: C) -> Self
    where
        C: IntoIterator<Item = D>,
        D: Into<BodyNode<'a>>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<BodyNode<'a>>,
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
    pub fn attribute(mut self, k: &'a str, v: &'a str) -> Self {
        self.attrs.push((k, v));
        self
    }
}

impl<'a> Display for Div<'a> {
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
        impl<'a> From<$name<'a>> for $enum_name<'a> {
            fn from(t: $name) -> $enum_name {
                $enum_name::$name(t)
            }
        }
    };
}

into_grouping_union!(Div, BodyNode);

macro_rules! into_grouping_union_without_lifetimes {
    ($name:ident, $enum_name:ident) => {
        impl<'a> From<$name> for $enum_name<'a> {
            fn from(t: $name) -> $enum_name<'a> {
                $enum_name::$name(t)
            }
        }
    };
}

into_grouping_union_without_lifetimes!(Br, BodyNode);
into_grouping_union_without_lifetimes!(Br, FormNode);

into_grouping_union!(Meta, HeadNode);
into_grouping_union!(Title, HeadNode);

into_grouping_union!(A, BodyNode);

enum_display!(BodyNode, H1, H2, H3, H4, H5, H6, P, Br, Text, Form, Div, A);

#[derive(Default, Debug, Clone)]
pub struct H1<'a>(Cow<'a, str>);

impl_of_heading_new_fn!(H1);

into_grouping_union!(H1, BodyNode);

heading_display!(H1);

#[derive(Default, Debug, Clone)]
pub struct H2<'a>(Cow<'a, str>);

impl_of_heading_new_fn!(H2);

into_grouping_union!(H2, BodyNode);

heading_display!(H2);

#[derive(Default, Debug, Clone)]
pub struct H3<'a>(Cow<'a, str>);

impl_of_heading_new_fn!(H3);

into_grouping_union!(H3, BodyNode);

heading_display!(H3);

#[derive(Default, Debug, Clone)]
pub struct H4<'a>(Cow<'a, str>);

impl_of_heading_new_fn!(H4);

into_grouping_union!(H4, BodyNode);

heading_display!(H4);

#[derive(Default, Debug, Clone)]
pub struct H5<'a>(Cow<'a, str>);

impl_of_heading_new_fn!(H5);

into_grouping_union!(H5, BodyNode);

heading_display!(H5);

#[derive(Default, Debug, Clone)]
pub struct H6<'a>(Cow<'a, str>);

impl_of_heading_new_fn!(H6);

into_grouping_union!(H6, BodyNode);

heading_display!(H6);

#[derive(Default, Debug, Clone)]
pub struct P<'a> {
    children: Vec<BodyNode<'a>>,
}

into_grouping_union!(P, BodyNode);

impl<'a> Display for P<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<p>")?;
        for child in &self.children {
            child.fmt(f)?;
        }
        f.write_str("</p>")
    }
}

impl<'a> P<'a> {
    pub fn children<C>(mut self, children: Vec<C>) -> Self
    where
        C: Into<BodyNode<'a>>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<BodyNode<'a>>,
    {
        self.children.push(child.into());
        self
    }
    pub fn with_text<S>(text: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        P::default().child(BodyNode::Text(Text::new(text)))
    }
    pub fn text<S>(self, text: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.child(BodyNode::Text(Text(text.into())))
    }
}

#[derive(Default, Debug, Clone)]
pub struct Text<'a>(Cow<'a, str>);

impl_of_heading_new_fn!(Text);

into_grouping_union!(Text, BodyNode);

impl<'a> Display for Text<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Form<'a> {
    children: Vec<FormNode<'a>>,
    attrs: Vec<(&'a str, &'a str)>,
}

impl<'a> Form<'a> {
    #[inline(always)]
    pub fn children<C>(mut self, children: Vec<C>) -> Self
    where
        C: Into<FormNode<'a>>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
    #[inline(always)]
    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<FormNode<'a>>,
    {
        self.children.push(child.into());
        self
    }
    #[inline(always)]
    pub fn attribute(mut self, k: &'a str, v: &'a str) -> Self {
        self.attrs.push((k, v));
        self
    }
}

impl<'a> Display for Form<'a> {
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
pub enum FormNode<'a> {
    Input(Input<'a>),
    Label(Label<'a>),
    Br(Br),
}

enum_display!(FormNode, Input, Label, Br);

#[derive(Debug, Clone, Default)]
pub struct Input<'a> {
    attrs: Vec<(Cow<'a, str>, Cow<'a, str>)>,
}

impl<'a> Display for Input<'a> {
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

impl<'a> Input<'a> {
    #[inline(always)]
    pub fn attribute<S1, S2>(mut self, k: S1, v: S2) -> Self
    where
        S1: Into<Cow<'a, str>>,
        S2: Into<Cow<'a, str>>,
    {
        self.attrs.push((k.into(), v.into()));
        self
    }
}

#[derive(Debug, Clone)]
pub struct Label<'a>(&'a str);

heading_display!(Label);

into_grouping_union!(Label, FormNode);
