/*
This source code file is distributed subject to the terms of the Mozilla Public License v2.0.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
use crate::{
    enum_display,
    tags::{meta::Meta, style::StyleTag, title::Title},
    utility_enum,
};

#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
use crate::into_vnode_for_grouping_enum;

utility_enum!(
    #[allow(missing_docs)]
    /// A node which can be attached to the <head> tag.
    pub enum HeadNode {
        Title(Title),
        Meta(Meta),
        StyleTag(StyleTag),
    }
);
#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
into_vnode_for_grouping_enum!(HeadNode, Title, Meta, StyleTag);

enum_display!(HeadNode, Title, Meta, StyleTag);
