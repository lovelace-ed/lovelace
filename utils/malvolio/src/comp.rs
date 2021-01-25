//! Allows the mounting of child components to `BodyNode`'s.

use yew::virtual_dom::VNode;

use crate::into_vnode::IntoVNode;

impl IntoVNode for VNode {
    fn into_vnode(self) -> VNode {
        self
    }
}

impl From<yew::virtual_dom::VNode> for crate::tags::body::body_node::BodyNode {
    fn from(comp: yew::virtual_dom::VNode) -> Self {
        Self::VNode(comp)
    }
}

/// Turns a component into a `BodyNode` so that it can be added as a child to a body.
///
/// For the moment properties are not supported – if you need to do that, you can just use the
/// `html!` macro as an argument to the `child` function of the relevant tag (e.g.
/// `Body::new().child(html! { <SomeComponent prop1="x">}`).
pub fn comp<C>() -> yew::virtual_dom::VNode
where
    C: yew::Component,
    C::Properties: From<()>,
{
    yew::virtual_dom::VNode::VComp(yew::virtual_dom::VComp::new::<C>(
        From::from(()),
        yew::NodeRef::default(),
        None,
    ))
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);
    use yew::prelude::*;

    use crate::component_named_app_with_html;

    #[wasm_bindgen_test]
    fn test_mount_component() {
        component_named_app_with_html!(crate::prelude::H1::new("Heading1")
            .attribute(crate::prelude::Id::new("some-heading"))
            .to_html());
        struct AppAcceptor {}
        impl Component for AppAcceptor {
            type Message = ();

            type Properties = ();

            fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
                Self {}
            }

            fn update(&mut self, _msg: Self::Message) -> ShouldRender {
                false
            }

            fn change(&mut self, _props: Self::Properties) -> ShouldRender {
                false
            }

            fn view(&self) -> yew::virtual_dom::VNode {
                crate::prelude::Div::new()
                    .child(super::comp::<App>())
                    .to_html()
            }
        }
        yew::initialize();

        let document = web_sys::window().unwrap().document().unwrap();
        let root = document
            .create_element("div")
            .expect("failed to create element");
        yew::App::<AppAcceptor>::new().mount(root.clone());
        let link = root
            .get_elements_by_tag_name("h1")
            .named_item("some-heading")
            .unwrap();
        assert_eq!(&link.text_content().expect("No text content"), "Heading1");
    }
    #[wasm_bindgen_test]
    fn test_feed_in_html_macro() {
        component_named_app_with_html!(crate::prelude::Div::new()
            .child(html! {
                <h1 id="some-heading">{"Some text"}</h1>
            })
            .to_html());
        yew::initialize();

        let document = web_sys::window().unwrap().document().unwrap();
        let root = document
            .create_element("div")
            .expect("failed to create element");
        yew::App::<App>::new().mount(root.clone());
        let link = root
            .get_elements_by_tag_name("h1")
            .named_item("some-heading")
            .unwrap();
        assert_eq!(&link.text_content().expect("No text content"), "Some text");
    }
}
