/*
This source code file is distributed subject to the terms of the Mozilla Public License v2.0.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
use crate::{
    enum_display,
    tags::{meta::Meta, title::Title},
    utility_enum,
};

#[cfg(feature = "with_yew")]
use crate::into_vnode_for_grouping_enum;

utility_enum!(
    #[allow(missing_docs)]
    /// A node which can be attached to the <head> tag.
    pub enum HeadNode {
        Title(Title),
        Meta(Meta),
    }
);
#[cfg(feature = "with_yew")]
into_vnode_for_grouping_enum!(HeadNode, Title, Meta);

enum_display!(HeadNode, Title, Meta);
