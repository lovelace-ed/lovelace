/*
This source code file is distributed subject to the terms of the Mozilla Public License v2.0.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
//! A small set of data types for producing HTML code.
#![allow(clippy::useless_format)]

use std::{borrow::Cow, collections::HashMap, fmt::Display};

#[cfg(feature = "with_rocket")]
use std::io::Cursor;

#[cfg(feature = "with_rocket")]
use rocket::{response::Responder, Response};
#[cfg(feature = "with_yew")]
use yew::virtual_dom::Listener;

#[cfg(feature = "with_yew")]
use std::rc::Rc;

macro_rules! heading_display {
    ($name:ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("<")?;
                f.write_str(stringify!($name))?;
                f.write_str(" ")?;
                write_attributes(
                    #[cfg(feature = "with_yew")]
                    &self.2,
                    #[cfg(not(feature = "with_yew"))]
                    &self.1,
                    f,
                )?;
                f.write_str(">")?;
                self.0.fmt(f)?;
                f.write_str("</")?;
                f.write_str(stringify!($name))?;
                f.write_str(">")
            }
        }
    };
}

macro_rules! to_html {
    () => {
        #[cfg(feature = "with_yew")]
        pub fn to_html(self) -> yew::virtual_dom::VNode {
            IntoVNode::into(self)
        }
    };
}

#[cfg(feature = "with_yew")]
macro_rules! into_vnode_for_grouping_enum {
    ($name:ident, $($variant:ident),*) => {
        impl IntoVNode for $name {
            fn into(self) -> yew::virtual_dom::VNode {
                match self {
                    $(
                        Self::$variant(x) => {IntoVNode::into(x)}
                    ),*

                }
            }
        }
    };
}

macro_rules! add_single_attribute {
    ($lifetime:tt) => {
        #[inline(always)]
        pub fn attribute(mut self, k: & $lifetime str, v: & $lifetime str) -> Self {
            self.attrs.push((k, v));
            self
        }
    };
}

macro_rules! impl_of_data_struct_insert {
    () => {
        #[inline(always)]
        pub fn attribute<S1, S2>(mut self, k: S1, v: S2) -> Self
        where
            S1: Into<&'static str>,
            S2: Into<String>,
        {
            // all these features are probably going to come back to bite :-)
            self.attrs.insert(k.into(), v.into());
            self
        }
    };
}

macro_rules! impl_of_heading_insert {
    () => {
        #[inline(always)]
        pub fn attribute<S1, S2>(mut self, k: S1, v: S2) -> Self
        where
            S1: Into<&'static str>,
            S2: Into<String>,
        {
            // all these features are probably going to come back to bite :-)
            #[cfg(feature = "with_yew")]
            self.2.insert(k.into(), v.into());
            #[cfg(not(feature = "with_yew"))]
            self.1.insert(k.into(), v.into());
            self
        }
    };
}

macro_rules! impl_of_heading_new_fn {
    ($name:ident) => {
        impl $name {
            pub fn new<S>(from: S) -> Self
            where
                S: Into<std::borrow::Cow<'static, str>>,
            {
                Self(
                    from.into(),
                    #[cfg(feature = "with_yew")]
                    vec![],
                    std::collections::HashMap::new(),
                )
            }
            impl_of_heading_insert!();
        }
    };
}

#[cfg(feature = "with_yew")]
macro_rules! heading_of_vnode {
    ($name:ident) => {
        impl IntoVNode for $name {
            fn into(self) -> ::yew::virtual_dom::VNode {
                let mut vtag = ::yew::virtual_dom::VTag::new(stringify!($name));
                for (k, v) in self.2.into_iter() {
                    vtag.add_attribute(k, v);
                }
                vtag.add_child(::yew::virtual_dom::VText::new(self.0).into());
                vtag.into()
            }
        }
    };
}

#[cfg(feature = "with_yew")]
pub trait IntoVNode {
    fn into(self) -> ::yew::virtual_dom::VNode;
}

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

#[derive(Clone, Debug)]
pub struct Html {
    #[cfg(feature = "with_rocket")]
    status: u16,
    #[cfg(feature = "with_rocket")]
    reason: Option<&'static str>,
    head: Head,
    body: Body,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Html {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut tag = yew::virtual_dom::VTag::new("html");
        tag.add_children(vec![IntoVNode::into(self.head), IntoVNode::into(self.body)]);
        tag.into()
    }
}

impl Default for Html {
    fn default() -> Self {
        Self {
            #[cfg(feature = "with_rocket")]
            status: 200,
            #[cfg(feature = "with_rocket")]
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

#[cfg(feature = "with_rocket")]
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
    #[cfg(feature = "with_rocket")]
    pub fn status(mut self, code: u16) -> Self {
        self.status = code;
        self
    }
    #[cfg(feature = "with_rocket")]
    pub fn status_reason(mut self, reason: &'static str) -> Self {
        self.reason = Some(reason);
        self
    }
    to_html!();
}

#[derive(Default, Debug, Clone)]
pub struct Head {
    children: Vec<HeadNode>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Head {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut tag = yew::virtual_dom::VTag::new("head");
        tag.add_children(self.children.into_iter().map(IntoVNode::into));
        tag.into()
    }
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

#[cfg(feature = "with_yew")]
into_vnode_for_grouping_enum!(HeadNode, Title, Meta);

#[derive(Default, Debug, Clone)]
pub struct Meta {
    attrs: Vec<(&'static str, &'static str)>,
}

impl Meta {
    #[inline(always)]
    pub fn attribute(mut self, k: &'static str, v: &'static str) -> Self {
        self.attrs.push((k, v));
        self
    }
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Meta {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("meta");
        for (a, b) in self.attrs {
            vtag.add_attribute(a, b.to_string())
        }
        vtag.into()
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
pub struct Title(
    Cow<'static, str>,
    #[cfg(feature = "with_yew")] Vec<Rc<dyn Listener>>,
    HashMap<&'static str, String>,
);

#[cfg(feature = "with_yew")]
heading_of_vnode!(Title);

impl_of_heading_new_fn!(Title);

heading_display!(Title);

#[derive(Default, Debug, Clone)]
pub struct Body {
    children: Vec<BodyNode>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Body {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("body");
        vtag.add_children(self.children.into_iter().map(IntoVNode::into));
        vtag.into()
    }
}

impl Body {
    pub fn children<I, C>(mut self, children: I) -> Self
    where
        C: Into<BodyNode>,
        I: IntoIterator<Item = C>,
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
                    $(Self::$variant(x) => <$variant as std::fmt::Display>::fmt(&x.clone(), f)),*
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
    Input(Input),
    Label(Label),
    Select(Select),
}

#[cfg(feature = "with_yew")]
into_vnode_for_grouping_enum!(
    BodyNode, H1, H2, H3, H4, H5, H6, P, Br, Text, Form, Div, A, Input, Label, Select
);

#[derive(Debug, Clone, Default)]
pub struct A {
    attrs: Vec<(&'static str, &'static str)>,
    text: Cow<'static, str>,
    href: Cow<'static, str>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for A {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vnode = yew::virtual_dom::VTag::new("a");
        vnode.add_attribute("href", self.href);
        for (a, b) in self.attrs {
            vnode.add_attribute(a, b.to_string())
        }
        vnode.add_child(yew::virtual_dom::VText::new(String::from(self.text)).into());
        vnode.into()
    }
}

impl A {
    pub fn new<S>(href: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self {
            href: href.into(),
            ..Default::default()
        }
    }
    pub fn target(mut self, target: &'static str) -> Self {
        self.attrs.push(("target", target));
        self
    }
    pub fn text<S>(mut self, text: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.text = text.into();
        self
    }
    add_single_attribute!('static);
    to_html!();
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
        f.write_str("href=\"")?;
        self.href.fmt(f)?;
        f.write_str("\"")?;
        f.write_str(">")?;
        self.text.fmt(f)?;
        f.write_str("</a>")
    }
}

#[derive(Debug, Clone, Default)]
pub struct Div {
    children: Vec<BodyNode>,
    attrs: Vec<(&'static str, &'static str)>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Div {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("div");
        vtag.add_children(self.children.into_iter().map(IntoVNode::into));
        for (a, b) in self.attrs {
            vtag.add_attribute(a, b.to_string())
        }
        vtag.into()
    }
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
    add_single_attribute!('static);
    to_html!();
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

#[cfg(feature = "with_yew")]
impl IntoVNode for Br {
    fn into(self) -> yew::virtual_dom::VNode {
        yew::virtual_dom::VTag::new("br").into()
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

macro_rules! into_grouping_union_without_lifetimes {
    ($name:ident, $enum_name:ident) => {
        impl From<$name> for $enum_name {
            fn from(t: $name) -> $enum_name {
                $enum_name::$name(t)
            }
        }
    };
}

into_grouping_union_without_lifetimes!(Br, BodyNode);

into_grouping_union!(Meta, HeadNode);
into_grouping_union!(Title, HeadNode);

into_grouping_union!(A, BodyNode);

enum_display!(BodyNode, H1, H2, H3, H4, H5, H6, P, Br, Text, Form, Div, A, Input, Select, Label);

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

#[derive(Default, Debug, Clone)]
pub struct P {
    children: Vec<BodyNode>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for P {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("br");
        vtag.add_children(self.children.into_iter().map(IntoVNode::into));
        vtag.into()
    }
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
    pub fn with_text<S>(text: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        P::default().child(BodyNode::Text(Text::new(text)))
    }
    pub fn text<S>(self, text: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.child(BodyNode::Text(Text::new(text)))
    }
}

#[derive(Default, Debug, Clone)]
pub struct Text(
    Cow<'static, str>,
    #[cfg(feature = "with_yew")] Vec<Rc<dyn Listener>>,
    HashMap<&'static str, String>,
);

impl_of_heading_new_fn!(Text);

#[cfg(feature = "with_yew")]
heading_of_vnode!(Text);

into_grouping_union!(Text, BodyNode);

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Form {
    children: Vec<BodyNode>,
    attrs: Vec<(&'static str, &'static str)>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Form {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("form");
        vtag.add_children(self.children.into_iter().map(IntoVNode::into));
        for (a, b) in self.attrs {
            vtag.add_attribute(a, b.to_string())
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
    add_single_attribute!('static);
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

#[derive(Debug, Clone, Default)]
pub struct Input {
    attrs: Vec<(&'static str, Cow<'static, str>)>,
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
            .find(|item| item.0 == "type")
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
    pub fn attribute<S1, S2>(mut self, k: S1, v: S2) -> Self
    where
        S1: Into<&'static str>,
        S2: Into<Cow<'static, str>>,
    {
        self.attrs.push((k.into(), v.into()));
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
}

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

#[derive(Default, Debug, Clone)]
pub struct Select {
    attrs: HashMap<&'static str, String>,
    children: Vec<SelectOption>,
}

impl Select {
    impl_of_data_struct_insert!();
    pub fn children<I, C>(mut self, children: I) -> Self
    where
        C: Into<SelectOption>,
        I: IntoIterator<Item = C>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<SelectOption>,
    {
        self.children.push(child.into());
        self
    }
}

into_grouping_union!(Select, BodyNode);

impl Display for Select {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<select ")?;
        write_attributes(&self.attrs, f)?;
        f.write_str(">")?;
        for child in &self.children {
            child.fmt(f)?;
        }
        f.write_str("</select>")
    }
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Select {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("select");
        write_attributes_to_vtag(self.attrs, &mut vtag);
        vtag.add_children(self.children.into_iter().map(IntoVNode::into));
        vtag.into()
    }
}

#[derive(Default, Debug, Clone)]
pub struct SelectOption {
    attrs: HashMap<&'static str, String>,
    text: Cow<'static, str>,
}

impl SelectOption {
    impl_of_data_struct_insert!();
    pub fn text<S>(mut self, text: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.text = text.into();
        self
    }
}

impl Display for SelectOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<option ")?;
        write_attributes(&self.attrs, f)?;
        f.write_str(">")?;
        self.text.fmt(f)?;
        f.write_str("</option>")
    }
}

#[cfg(feature = "with_yew")]
impl IntoVNode for SelectOption {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("option");
        write_attributes_to_vtag(self.attrs, &mut vtag);
        vtag.add_child(::yew::virtual_dom::VText::new(self.text).into());
        vtag.into()
    }
}

#[cfg(feature = "yew")]
fn write_attributes_to_vtag(
    attrs: HashMap<&'static str, String>,
    vtag: &mut ::yew::virtual_dom::VTag,
) {
    for (key, value) in attrs.into_iter() {
        vtag.add_attribute(key, value);
    }
}

fn write_attributes(
    attrs: &HashMap<&'static str, String>,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    for (key, value) in attrs {
        f.write_str(key)?;
        f.write_str("=\"")?;
        f.write_str(&value)?;
        f.write_str("\"")?;
    }
    Ok(())
}
