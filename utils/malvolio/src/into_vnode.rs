#[cfg(feature = "with_yew")]
pub trait IntoVNode {
    fn into_vnode(self) -> ::yew::virtual_dom::VNode;
}
