/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
#![deny(missing_debug_implementations)]

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
use portia::levels::Level;
use rocket::Rocket;
use utils::launch;

mod auth;
mod calendar;
mod class;
mod css_names;
mod db;
mod email;
mod models;
mod notifications;
mod schema;
mod utils;

#[derive(CSS, Debug)]
#[elements(H1, H2, H3, H4, H5, H6)]
#[font_family = "sans-serif"]
#[font_size = "24px"]
pub struct TitleStyles;

#[derive(CSS, Debug)]
#[elements(Div)]
#[background_color = "#f4d03f"]
pub struct YellowBackground;

#[derive(CSS, Debug)]
#[elements(Div)]
#[background_color = "#d5dbdb"]
pub struct GreyBackground;

#[derive(CSS, Debug)]
#[elements(Body)]
#[margin = "0"]
pub struct ZeroMargin;

#[derive(CSS, Debug)]
#[elements(Div)]
#[padding = "15px"]
pub struct DefaultPadding;

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
                        .child(H1::new("Welcome!").apply(TitleStyles)))
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

#[rocket::launch]
fn rocket() -> Rocket {
    launch()
}
