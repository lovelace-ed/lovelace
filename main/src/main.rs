/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
#![deny(missing_debug_implementations, unused_must_use, unused_mut)]

#[macro_use]
extern crate serde;
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
#[macro_use]
extern crate cfg_if;
extern crate jsonwebtoken as jwt;

use malvolio::prelude::{Body, Content, Div, Head, Href, Html, Meta, MetaName, A, H1, P};
use mercutio::*;
use portia::{
    colour::{GreyBackground, YellowBackground},
    font::SmallTitle,
    levels::Level,
    margin::ZeroMargin,
    padding::DefaultPadding,
};

mod auth;
mod calendar;
mod class;
mod dashboard;
mod db;
mod email;
mod institution;
mod models;
mod notifications;
mod schema;
mod utils;

#[get("/")]
fn index() -> Html {
    Html::default()
        .head(
            Head::default().child(
                Meta::default()
                    .attribute(MetaName::Charset)
                    .attribute(Content::new("utf-8")),
            ),
        )
        .body(
            Body::new().apply(ZeroMargin)
            .child(
                Level::new()
                    .child(Div::new()
                        .apply(compose(YellowBackground, DefaultPadding))
                        .child(H1::new("Welcome!").apply(SmallTitle)))
                    .child(Level::new()
                        .child(P::with_text(
                            "IMPORTANT: This site is in beta. Please do not input any data
                            into it yet (we have hidden all the buttons away for the meantime, until we can be
                            confident that they're safe to press :)",
                        ))
                        .child(P::with_text(
                            "Lovelace is a digital platform for learning. It's also quite an
                            incomplete one at the moment, but we're adding features relatively quickly. Updates
                            to this site are rolled out on a weekly basis, so check back soon for more.",
                        ))
                        .into_div()
                        .apply(compose(GreyBackground, DefaultPadding))
                    )
                    .child(
                        Level::new().child(
                            A::default()
                                .attribute(Href::new("https://github.com/lovelace-ed/lovelace"))
                                .text("Click me to view the source code.")
                        )
                        .into_div()
                        .apply(DefaultPadding)
                    )
                )
            )
}

#[tokio::main]

async fn main() {
    crate::utils::launch().launch().await.unwrap();
}
