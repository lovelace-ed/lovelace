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

use html::{Body, Head, Html, Meta, H1};
use utils::launch;

mod auth;
mod class;
mod db;
mod models;
mod schema;
mod utils;

#[get("/")]
fn index() -> Html {
    Html::default()
        .head(
            Head::default()
                .child(Meta::default().attribute(format!("charset"), format!("UTF-8")))
                .child(Meta::default().attribute(format!("lang"), format!("en-GB"))),
        )
        .body(Body::default().child(H1(format!("Hello World!"))))
}

fn main() {
    launch().launch();
}
