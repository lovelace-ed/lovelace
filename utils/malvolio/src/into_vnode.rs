#[cfg(feature = "with_yew")]
/// Used to convert things into `VNode`'s.
pub trait IntoVNode {
    /// Convert the current item into a `VNode`.
    fn into_vnode(self) -> ::yew::virtual_dom::VNode;
}
