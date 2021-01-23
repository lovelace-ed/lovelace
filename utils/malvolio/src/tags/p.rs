use std::{borrow::Cow, fmt::Display};

use super::body::body_node::BodyNode;
#[cfg(feature = "with_yew")]
use crate::into_vnode::IntoVNode;
use crate::{into_grouping_union, text::Text};
use ammonia::clean;

#[derive(Default, Debug, Clone)]
pub struct P {
    children: Vec<BodyNode>,
}

#[cfg(feature = "with_yew")]
impl IntoVNode for P {
    fn into(self) -> yew::virtual_dom::VNode {
        let mut vtag = yew::virtual_dom::VTag::new("br");
        vtag.add_children(self.children.into_iter().map(IntoVNode::into));
        vtag.into()
    }
}

into_grouping_union!(P, BodyNode);

impl Display for P {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<p>")?;
        for child in &self.children {
            child.fmt(f)?;
        }
        f.write_str("</p>")
    }
}

impl P {
    pub fn children<C>(mut self, children: Vec<C>) -> Self
    where
        C: Into<BodyNode>,
    {
        self.children
            .extend(children.into_iter().map(Into::into).collect::<Vec<_>>());
        self
    }
    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<BodyNode>,
    {
        self.children.push(child.into());
        self
    }
    pub fn with_text<S>(text: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        P::default().child(BodyNode::Text(Text::new(text)))
    }
    /// Adds the supplied text to this node, overwriting the previously existing text (if text has
    /// already been added to the node).
    ///
    /// This method sanitises the input (i.e. it escapes HTML);
    /// this might not be what you want – if you are *absolutely certain* that the text you are
    /// providing does not come from a potentially malicious source (e.g. user-supplied text can
    /// contain script tags which will execute unwanted code) you can use `text_unsanitized` which
    /// is identical to this method, except for that it does not sanitise the inputted text (and is
    /// thus slightly faster).
    pub fn text<S>(self, text: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.child(BodyNode::Text(Text::new(clean(&text.into()))))
    }
    /// Adds the supplied text to this node, overwriting the previously existing text (if text has
    /// already been added to the node).
    ///
    /// WARNING: Do not (under any circumstances) use this method with unescaped user-supplied text.
    /// It will be rendered and poses a major security threat to your application. If in doubt, use
    /// the `text` method instead of this one (the risk is much lower that way).
    pub fn text_unsanitized<S>(self, text: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.child(BodyNode::Text(Text::new(text)))
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    #[test]
    fn test_p() {
        let document = P::with_text("Some text").to_string();
        let document = scraper::Html::parse_document(&document);
        let p = scraper::Selector::parse("p").unwrap();
        let p = document.select(&p).next().unwrap();
        assert_eq!(
            p.children()
                .next()
                .unwrap()
                .value()
                .as_text()
                .unwrap()
                .to_string()
                .as_str(),
            "Some text"
        );
    }
}
