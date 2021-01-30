/*
This source code file is distributed subject to the terms of the Mozilla Public License v2.0.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
#[cfg(feature = "with_yew")]
#[cfg(not(tarpaulin))]
/// Used to convert things into `VNode`'s.
pub trait IntoVNode {
    /// Convert the current item into a `VNode`.
    fn into_vnode(self) -> ::yew::virtual_dom::VNode;
}
