//! A CalDAV client.
//!
//! Note: this crate is "asynchronous." If you're writing a synchronous application, then you should
//! use something like `futures::block_on` (futures is a crates.io/crates/futures).

// when you use too many macros (but `xml!` a nice DSL macro as far as DSL macros go)
#![recursion_limit = "256"]

#[macro_use]
extern crate thiserror;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate format_xml;

pub mod calendar;
pub mod client;
pub mod error;
pub mod event;
