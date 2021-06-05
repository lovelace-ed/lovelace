use crate::prelude::BodyNode;
use std::{borrow::Cow, fmt::Display};

use crate::into_grouping_union;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "pub_fields", derive(FieldsAccessibleVariant))]
/// The <noscript> tag. The contents of this tag will be shown to people whose browsers don't
/// support Javascript, or who don't have Javascript enabled.
///
/// See [MDN's page on this](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/noscript) for
/// further information.
pub struct NoScript {
    text: Cow<'static, str>,
}

/// Creates a new `NoScript` tag – functionally equivalent to `NoScript::new(<text>)` (but easier to
/// type.)
pub fn noscript(text: impl Into<Cow<'static, str>>) -> NoScript {
    NoScript::new(text)
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
