use std::{borrow::Cow, collections::HashSet};

use super::IntoAttribute;

#[derive(Debug, Derivative, Clone)]
#[derivative(Default(new = "true"))]
/// A builder for constructing values for the `class` attribute.
///
/// You should use this struct if you want to build a set of classes using information which is not
/// known at compile time. If you know all the classes you want to use beforehand you shold use
/// `StaticClasses` instead.
///
/// For example if you retrieve a set of notifications from a database and then want to apply the
/// "warning" class to all the notifications which are marked as warning you want this struct. If
/// you just want to render a list of notifications with the class "notification" then that doesn't
/// depend on any information which you know only when the program is running, so you should use
/// `StaticClasses` for that.
pub struct Class(HashSet<Cow<'static, str>>);

impl From<&'static str> for Class {
    fn from(str: &'static str) -> Self {
        let mut set = HashSet::new();
        set.insert(str.into());
        Self(set)
    }
}

impl Class {
    pub fn class(mut self, class: Cow<'static, str>) -> Self {
        self.0.insert(class);
        self
    }
}

impl IntoAttribute for Class {
    fn into_attribute(self) -> (&'static str, Cow<'static, str>) {
        (
            "class",
            self.0.into_iter().collect::<Vec<_>>().join(" ").into(),
        )
    }
}

#[derive(Debug, Default, Clone)]
pub struct Id(Cow<'static, str>);

impl Id {
    pub fn new<C>(c: C) -> Self
    where
        C: Into<Cow<'static, str>>,
    {
        Self(c.into())
    }
}

impl IntoAttribute for Id {
    fn into_attribute(self) -> (&'static str, Cow<'static, str>) {
        ("id", self.0)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Style(Cow<'static, str>);

impl Style {
    pub fn new<C>(string: C) -> Self
    where
        C: Into<Cow<'static, str>>,
    {
        Self(string.into())
    }
}

impl IntoAttribute for Style {
    fn into_attribute(self) -> (&'static str, Cow<'static, str>) {
        ("style", self.0)
    }
}
