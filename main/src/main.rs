/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
#![feature(proc_macro_hygiene, decl_macro)]
#![allow(clippy::useless_format)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate log;

#[macro_use]
extern crate diesel_migrations;

#[macro_use]
extern crate rocket_contrib;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate nanoid;

#[macro_use]
extern crate derive_builder;

use malvolio::{Body, Head, Html, Meta, H1};
use utils::launch;

mod auth;
mod class;
mod css_names;
mod db;
mod email;
mod models;
mod notifications;
mod schema;
mod utils;

#[get("/")]
fn index() -> Html {
    Html::default()
        .head(
            Head::default()
                .child(Meta::default().attribute("charset", "UTF-8"))
                .child(Meta::default().attribute("lang", "en-GB")),
        )
        .body(Body::default().child(H1::new("Hello World!")))
}

fn main() {
    launch().launch();
}
