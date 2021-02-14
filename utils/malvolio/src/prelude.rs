/*
This source code file is distributed subject to the terms of the Mozilla Public License v2.0.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
pub use crate::tags::{
    a::{Href, A},
    body::body_node::BodyNode,
    body::Body,
    br::Br,
    div::Div,
    form::{Action, Form, Method},
    head::Head,
    headings::{H1, H2, H3, H4, H5, H6},
    html::Html,
    input::{Input, Name, Placeholder, Type, Value},
    label::Label,
    meta::{Content, Meta, MetaName},
    option::SelectOption,
    p::P,
    select::Select,
    style::StyleTag,
    title::Title,
};

pub use crate::attributes::common::{Class, Id, Style};
