#[cfg(feature = "with_yew")]
pub trait IntoVNode {
    fn into(self) -> ::yew::virtual_dom::VNode;
}
