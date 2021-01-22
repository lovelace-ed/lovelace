pub mod common;

use std::borrow::Cow;

pub trait IntoAttribute {
    fn into_attribute(self) -> (&'static str, Cow<'static, str>);
}
