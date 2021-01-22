use crate::{
    enum_display,
    tags::{
        a::A,
        br::Br,
        div::Div,
        form::Form,
        headings::{H1, H2, H3, H4, H5, H6},
        input::Input,
        label::Label,
        p::P,
        select::Select,
    },
    text::Text,
    utility_enum,
};

#[cfg(feature = "with_yew")]
use crate::into_vnode_for_grouping_enum;

utility_enum!(
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
);

#[cfg(feature = "with_yew")]
into_vnode_for_grouping_enum!(
    BodyNode, H1, H2, H3, H4, H5, H6, P, Br, Text, Form, Div, A, Input, Label, Select
);

enum_display!(BodyNode, H1, H2, H3, H4, H5, H6, P, Br, Text, Form, Div, A, Input, Select, Label);
