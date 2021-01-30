/*
This source code file is distributed subject to the terms of the Mozilla Public License v2.0.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
//! Malvolio is a library for writing declarative, strongly-typed HTML. There are many excellent and
//! thoughtful approaches people have developed for producing HTML code inside of Rust programs.
//! Malvolio integrates well (we think) into the rest of the Rust language and allows you to produce
//! strongly-typed HTML.
//!
//! Malvolio is "isomorphic," which is to say that you can use it both on servers and in browsers.
//!
//! Although in early stages, Malvolio works and is suitable (we think) for general use. We
//! anticipate some (but not major) breaking API changes will come as we add support for CSS
//! (defined in Rust, rather than externally).
//!
//! **Some self promotion:**
//!
//! We welcome contributions, issues, concerns and suggestions (if you just want to chat, join us in
//! the #malvolio channel on the [Yew Discord server](https://discord.gg/VQck8X4))! Malvolio's
//! source code is located in the `utils/malvolio` directory of the
//! [Lovelace repository](https://github.com/lovelace-ed/lovelace). We're happy to mentor new
//! contributors if they need help.
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
pub(crate) mod macros;
pub(crate) mod utils;
#[macro_use]
pub(crate) mod docs;
