use std::fmt::Display;

use crate::{into_grouping_union_without_lifetimes, to_html};
#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;

use super::body::body_node::BodyNode;
#[derive(Debug, Clone)]
/// A new line.
///
/// ```
/// # use malvolio::prelude::*;
/// Div::new().child(Br);
/// ```
///
/// See the [MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/br) for more
/// info.
pub struct Br;

impl Br {
    to_html!();
}

impl Display for Br {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<br/>")
    }
}

#[cfg(feature = "with_yew")]
impl IntoVNode for Br {
    fn into(self) -> yew::virtual_dom::VNode {
        yew::virtual_dom::VTag::new("br").into()
    }
}

into_grouping_union_without_lifetimes!(Br, BodyNode);

#[cfg(test)]
mod test {
    use crate::prelude::*;
    #[test]
    fn test_br() {
        let document = Br.to_string();
        let document = scraper::Html::parse_document(&document);
        let br = scraper::Selector::parse("br").unwrap();
        document.select(&br).next().unwrap();
    }
}
