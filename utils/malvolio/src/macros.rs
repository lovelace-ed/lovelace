/*
This source code file is distributed subject to the terms of the Mozilla Public License v2.0.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
//! Important: these are not intended for general consumption (only use them internally).
//!
//! A set of macros which are used to reduce the number of times one has to type out the same thing
//! over and over again, which I assure you is very boring (repeated typing of the same thing over
//! and over again tends to lead to asking existential questions as a way to pass the time – I'm
//! rambling here, aren't I :)

#[macro_export]
/// For internal use only.
macro_rules! heading_display {
    ($name:ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("<")?;
                f.write_str(stringify!($name))?;
                f.write_str(" ")?;
                crate::utils::write_attributes(&self.1, f)?;
                f.write_str(">")?;
                self.0.fmt(f)?;
                f.write_str("</")?;
                f.write_str(stringify!($name))?;
                f.write_str(">")
            }
        }
    };
}

#[macro_export]
/// For internal case only.
///
/// Generates new code to construct a heading.
macro_rules! impl_of_heading_new_fn {
    ($name:ident) => {
        impl $name {
            /// Create a new item of this type, given an item which can be converted into a
            /// `Cow<'static, str>` (for example a `&str` or a `String`).
            pub fn new<S>(from: S) -> Self
            where
                S: Into<std::borrow::Cow<'static, str>>,
            {
                Self(
                    from.into(),
                    std::collections::HashMap::new(),
                    #[cfg(feature = "with_yew")]
                    vec![],
                )
            }
            #[inline(always)]
            /// Attach a new attribute to this node.
            pub fn attribute<A>(mut self, a: A) -> Self
            where
                A: Into<$crate::tags::headings::HeadingAttr>,
            {
                use $crate::attributes::IntoAttribute;
                let (a, b) = a.into().into_attribute();
                self.1.insert(a, b);
                self
            }
        }
    };
}

#[cfg(feature = "with_yew")]
#[macro_export]
/// For internal use only.
macro_rules! heading_of_vnode {
    ($name:ident) => {
        impl $crate::into_vnode::IntoVNode for $name {
            fn into_vnode(self) -> ::yew::virtual_dom::VNode {
                let mut vtag = ::yew::virtual_dom::VTag::new(stringify!($name));
                for (k, v) in self.1.into_iter() {
                    vtag.add_attribute(k, &v);
                }
                vtag.add_child(::yew::virtual_dom::VText::new(self.0.to_string()).into());
                vtag.into()
            }
        }
        impl $name {
            $crate::to_html!();
        }
    };
}

#[macro_export]
/// For internal use only.
macro_rules! enum_display {
    ($on:ident, $($variant:ident),*) => {
        impl std::fmt::Display for $on {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$variant(x) => <$variant as std::fmt::Display>::fmt(&x.clone(), f)),*,
                    #[allow(unreachable_patterns)]
                    _ => panic!("Virtual components are not supported.")
                }
            }
        }
    };
}

#[macro_export]
/// For internal use only.
macro_rules! into_grouping_union {
    ($name:ident, $enum_name:ident) => {
        impl From<$name> for $enum_name {
            fn from(t: $name) -> $enum_name {
                $enum_name::$name(t)
            }
        }
    };
}

#[macro_export]
/// For intenal use only.
macro_rules! into_grouping_union_without_lifetimes {
    ($name:ident, $enum_name:ident) => {
        impl From<$name> for $enum_name {
            fn from(t: $name) -> $enum_name {
                $enum_name::$name(t)
            }
        }
    };
}

#[cfg(feature = "with_yew")]
#[macro_export]
/// For internal use only.
macro_rules! into_vnode_for_grouping_enum {
    ($name:ident, $($variant:ident),*) => {
        impl $crate::into_vnode::IntoVNode for $name {
            fn into_vnode(self) -> yew::virtual_dom::VNode {
                match self {
                    $(
                        Self::$variant(x) => {$crate::into_vnode::IntoVNode::into_vnode(x)}
                    ),*

                }
            }
        }
    };
}

#[macro_export]
/// For internal use only.
macro_rules! add_single_attribute {
    ($lifetime:tt) => {
        #[inline(always)]
        pub fn attribute(mut self, k: & $lifetime str, v: & $lifetime str) -> Self {
            self.attrs.push((k, v));
            self
        }
    };
}

#[macro_export]
/// Generates a function to call `into_vnode`
macro_rules! to_html {
    () => {
        #[cfg(feature = "with_yew")]
        /// Turn this item into a `VNode`. You only need to call this on the item that you
        /// return in the `html` function.
        pub fn to_html(self) -> yew::virtual_dom::VNode {
            $crate::into_vnode::IntoVNode::into_vnode(self)
        }
    };
}

#[macro_export]
/// Generates code to convert an attribute into a grouping enum.
///
/// For internal use only.
macro_rules! into_attribute_for_grouping_enum {
    ($name:ident, $($variant:ident),*) => {
        impl $crate::attributes::IntoAttribute for $name {
            fn into_attribute(self) -> (&'static str, std::borrow::Cow<'static, str>) {
                match self {
                    $(
                        Self::$variant(x) => {$crate::attributes::IntoAttribute::into_attribute(x)}
                    ),*

                }
            }
        }
    };
}

#[cfg(test)]
#[macro_export]
/// For internal use only
macro_rules! component_named_app_with_html {
    ($($html:tt)*) => {
        struct App {}
        impl Component for App {
            type Properties = ();
            type Message = ();
            fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
                Self {}
            }
            fn update(&mut self, _msg: Self::Message) -> bool {
                false
            }
            fn change(&mut self, _props: Self::Properties) -> bool {
                false
            }
            fn view(&self) -> ::yew::virtual_dom::VNode {
                $($html)*
            }
        }
    }
}
