use crate::{
    enum_display,
    tags::{meta::Meta, title::Title},
};

#[cfg(feature = "with_yew")]
use crate::into_vnode_for_grouping_enum;

#[derive(Debug, Clone)]
pub enum HeadNode {
    Title(Title),
    Meta(Meta),
}

#[cfg(feature = "with_yew")]
into_vnode_for_grouping_enum!(HeadNode, Title, Meta);

enum_display!(HeadNode, Title, Meta);
