/*
This source code file is distributed subject to the terms of the Mozilla Public License v2.0.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
//! A small set of data types for producing HTML code.
#![allow(clippy::useless_format)]

#[cfg(feature = "with_yew")]
pub mod into_vnode;
pub mod prelude;
pub mod tags;
pub mod text;

#[macro_use]
mod macros;
pub(crate) mod utils;
