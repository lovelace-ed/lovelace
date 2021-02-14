#[macro_use]
extern crate quote;
#[macro_use]
extern crate darling;
#[macro_use]
extern crate nanoid;

use proc_macro::TokenStream;
mod inner;

#[proc_macro_derive(CSS, attributes(mercutio))]
pub fn derive_css(input: TokenStream) -> TokenStream {
    inner::css_inner(syn::parse_macro_input!(input as syn::DeriveInput)).into()
}
