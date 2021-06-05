/*
This source code file is distributed subject to the terms of the Mozilla Public License v2.0.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
use std::{borrow::Cow, fmt::Display};

use ammonia::clean;

use super::body::body_node::BodyNode;
use crate::{into_grouping_union, text::Text};

/// The <p> tag.
///
/// See the [MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p) for more
/// info.
#[derive(Derivative, Debug, Clone)]
#[derivative(Default(new = "true"))]
#[cfg_attr(feature = "pub_fields", derive(FieldsAccessibleVariant))]

pub struct P {
    text: Cow<'static, str>,
    children: Vec<BodyNode>,
}

/// Creates a new `P` tag – functionally equivalent to `P::new()` (but easier to type.)
pub fn p() -> P {
    P::new()
}

into_grouping_union!(P, BodyNode);

impl Display for P {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<p>")?;
        self.text.fmt(f)?;
        for child in &self.children {
            child.fmt(f)?;
        }
        f.write_str("</p>")
    }
}

impl P {
    /// A method to construct a paragraph containing the supplied text. This will sanitise the text
    /// provided beforehand.
    pub fn with_text<S>(text: S) -> Self
    where
        S: AsRef<str>,
    {
        Self {
            text: clean(text.as_ref()).into(),
            children: vec![],
        }
    }

    /// Attach a new child to this tag.
    pub fn child(mut self, child: impl Into<BodyNode>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Add new children to this tag from an iterator.
    pub fn children(mut self, children: impl IntoIterator<Item = BodyNode>) -> Self {
        self.children.extend(children);
        self
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
        self.child(BodyNode::Text(Text::new(text.into())))
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
        self.child(BodyNode::Text(Text::new_unchecked(text.into())))
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
