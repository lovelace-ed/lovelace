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
extern crate rocket_sync_db_pools;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate nanoid;
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate cfg_if;
#[macro_use]
extern crate derivative;
extern crate jsonwebtoken as jwt;

mod auth;
mod calendar;
mod class;
mod dashboard;
mod db;
mod email;
mod home;
mod institution;
mod models;
mod notifications;
mod schema;
mod ui;
mod utils;

#[rocket::launch]
fn launch() -> _ {
    crate::utils::launch()
}
