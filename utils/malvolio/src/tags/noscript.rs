use crate::prelude::BodyNode;
use std::{borrow::Cow, fmt::Display};

use crate::into_grouping_union;
#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;

#[derive(Debug, Clone)]
/// The <noscript> tag.
pub struct NoScript {
    text: Cow<'static, str>,
}

impl NoScript {
    /// Construct a new <noscript> tag.
    pub fn new<T>(text: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Self { text: text.into() }
    }
}

impl Display for NoScript {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<noscript>")?;
        f.write_str(&self.text)?;
        f.write_str("</noscript>")
    }
}

#[cfg(feature = "with_yew")]
impl IntoVNode for NoScript {
    fn into_vnode(self) -> yew::virtual_dom::VNode {
        let mut vtag = ::yew::virtual_dom::VTag::new("noscript");
        vtag.add_child(::yew::virtual_dom::VText::new(self.text.to_string()).into());
        vtag.into()
    }
}

into_grouping_union!(NoScript, BodyNode);

#[cfg(test)]
mod test {
    use super::NoScript;

    #[test]
    fn test_noscript() {
        let document = NoScript::new("No Javascript :)").to_string();
        let document = scraper::Html::parse_document(&document);
        let noscript = scraper::Selector::parse("noscript").unwrap();
        let tag = document.select(&noscript).next().unwrap();
        assert_eq!(
            tag.first_child()
                .unwrap()
                .value()
                .as_text()
                .unwrap()
                .to_string()
                .as_str(),
            "No Javascript :)"
        );
    }
}
