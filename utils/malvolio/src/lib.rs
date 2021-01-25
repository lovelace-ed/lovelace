/*
This source code file is distributed subject to the terms of the Mozilla Public License v2.0.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
//! A small set of data types for producing HTML code.

#![deny(missing_docs, missing_debug_implementations)]

#[macro_use]
extern crate derivative;

/// Attributes which can be attached to multiple nodes.
pub mod attributes;
#[cfg(feature = "with_yew")]
pub mod comp;
#[cfg(feature = "with_yew")]
/// Contains a trait to allow you to convert items into `VNode`'s. You shouldn't need to use this,
/// but it is helpful so for some niche use cases which is why it's public.
pub mod into_vnode;
/// A list of types which are useful for using the library. Unless you have name conflicts, we
/// recommend just inserting a `use malvolio::prelude::*;`.
pub mod prelude;
/// The different HTML tags which Malvolio supports.
pub mod tags;
/// A text node.
pub mod text;
#[macro_use]
mod macros;
pub(crate) mod utils;
#[macro_use]
pub(crate) mod docs;
