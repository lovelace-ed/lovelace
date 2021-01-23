//! Important: these are not intended for general consumption (only use them internally).
//!
//! A set of macros which are used to reduce the number of times one has to type out the same thing
//! over and over again, which I assure you is very boring (repeated typing of the same thing over
//! and over again tends to lead to asking existential questions as a way to pass the time – I'm
//! rambling here, aren't I :)

#[macro_export]
#[cfg(feature = "with_yew")]
macro_rules! heading_display {
    ($name:ident) => {
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("<")?;
                f.write_str(stringify!($name))?;
                f.write_str(" ")?;
                crate::utils::write_attributes(&self.2, f)?;
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
#[cfg(not(feature = "with_yew"))]
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
macro_rules! impl_of_data_struct_insert {
    () => {
        #[inline(always)]
        pub fn attribute<S1, S2>(mut self, k: S1, v: S2) -> Self
        where
            S1: Into<&'static str>,
            S2: Into<String>,
        {
            // all these features are probably going to come back to bite :-)
            self.attrs.insert(k.into(), v.into());
            self
        }
    };
}

#[macro_export]
macro_rules! impl_of_heading_new_fn {
    ($name:ident) => {
        impl $name {
            pub fn new<S>(from: S) -> Self
            where
                S: Into<std::borrow::Cow<'static, str>>,
            {
                Self(
                    from.into(),
                    #[cfg(feature = "with_yew")]
                    vec![],
                    std::collections::HashMap::new(),
                )
            }
            #[inline(always)]
            pub fn attribute<S1, S2>(mut self, k: S1, v: S2) -> Self
            where
                S1: Into<&'static str>,
                S2: Into<String>,
            {
                // all these features are probably going to come back to bite :-)
                #[cfg(feature = "with_yew")]
                self.2.insert(k.into(), v.into());
                #[cfg(not(feature = "with_yew"))]
                self.1.insert(k.into(), v.into());
                self
            }
        }
    };
}

#[cfg(feature = "with_yew")]
#[macro_export]
macro_rules! heading_of_vnode {
    ($name:ident) => {
        impl $crate::into_vnode::IntoVNode for $name {
            fn into(self) -> ::yew::virtual_dom::VNode {
                let mut vtag = ::yew::virtual_dom::VTag::new(stringify!($name));
                for (k, v) in self.2.into_iter() {
                    vtag.add_attribute(k, &v);
                }
                vtag.add_child(::yew::virtual_dom::VText::new(self.0.to_string()).into());
                vtag.into()
            }
        }
    };
}

#[macro_export]
macro_rules! enum_display {
    ($on:ident, $($variant:ident),*) => {
        impl std::fmt::Display for $on {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$variant(x) => <$variant as std::fmt::Display>::fmt(&x.clone(), f)),*
                }
            }
        }
    };
}

#[macro_export]
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
macro_rules! into_vnode_for_grouping_enum {
    ($name:ident, $($variant:ident),*) => {
        impl $crate::into_vnode::IntoVNode for $name {
            fn into(self) -> yew::virtual_dom::VNode {
                match self {
                    $(
                        Self::$variant(x) => {$crate::into_vnode::IntoVNode::into(x)}
                    ),*

                }
            }
        }
    };
}

#[macro_export]
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
macro_rules! to_html {
    () => {
        #[cfg(feature = "with_yew")]
        pub fn to_html(self) -> yew::virtual_dom::VNode {
            IntoVNode::into(self)
        }
    };
}

#[macro_export]
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
