/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
use std::collections::HashMap;

pub mod auto_database_error;
pub mod error_messages;
pub mod html_or_redirect;
pub mod timezones;

#[cfg(test)]
use crate::auth::LOGIN_COOKIE;
use malvolio::prelude::{Body, Head, Html, Meta, Title, H1, P};
use rocket::{
    config::{Environment, Value},
    fairing::AdHoc,
    Config, Rocket,
};
#[cfg(test)]
use rocket::{http::ContentType, local::Client};

pub fn default_head(title: String) -> Head {
    Head::default()
        .child(Title::new(title + " | Lovelace"))
        .child(
            Meta::default()
                .attribute("rel", "stylesheet")
                .attribute("href", "/css/styles.css"),
        )
}

pub fn retrieve_database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://localhost".to_string())
}

pub fn launch() -> Rocket {
    let mut database_config = HashMap::new();
    let mut databases = HashMap::new();
    database_config.insert("url", Value::from(retrieve_database_url()));
    databases.insert("postgres", Value::from(database_config));
    let config = Config::build(
        std::env::var("DEV")
            .map(|_| Environment::Development)
            .unwrap_or(Environment::Production),
    )
    .secret_key(
        std::env::var("SECRET_KEY")
            .unwrap_or_else(|_| "NNnXxqFeQ/1Sn8lh9MtlIW2uePR4TL/1O5dB2CPkTmg=".to_string()),
    )
    .extra("databases", databases)
    .finalize()
    .unwrap();
    rocket::custom(config)
        .attach(crate::db::Database::fairing())
        .attach(AdHoc::on_attach(
            "Database Migrations",
            crate::db::run_migrations,
        ))
        .mount(
            "/",
            routes![
                crate::index,
                crate::class::create_class,
                crate::class::create_class_page,
                crate::class::join_class,
                crate::class::view_all_classes,
                crate::class::view_class_overview,
                crate::class::get_class_settings,
                crate::class::view_class_members_page,
                crate::class::invite_teacher_page,
                crate::class::invite_teacher,
                crate::class::delete_class_page,
                crate::class::delete_class,
                crate::auth::logout
            ],
        )
        .mount(
            "/auth",
            routes![
                crate::auth::login_page,
                crate::auth::login,
                crate::auth::register_page,
                crate::auth::register
            ],
        )
        .mount(
            "/notifications",
            routes![
                crate::notifications::list_notifications,
                crate::notifications::mark_notification_as_read,
                crate::notifications::delete_notification_with_id
            ],
        )
        .mount(
            "/class",
            routes![
                crate::class::messages::list_all_messages,
                crate::class::messages::create_new_class_message,
                crate::class::messages::apply_create_new_class_message,
                crate::class::messages::reply_to_teacher_message,
                crate::class::messages::edit_message,
                crate::class::messages::apply_message_edit,
                crate::class::messages::edit_message_reply,
                crate::class::messages::apply_message_reply_edit,
                crate::class::messages::view_message
            ],
        )
}

pub fn error_message(title: String, message: String) -> Html {
    Html::default().head(default_head(title.clone())).body(
        Body::default()
            .child(H1::new(title))
            .child(P::with_text(message)),
    )
}

#[cfg(test)]
pub fn client() -> Client {
    let rocket = launch();
    Client::new(rocket).expect("needs a valid rocket instance")
}

#[cfg(test)]
pub fn create_user(username: &str, email: &str, timezone: &str, password: &str, client: &Client) {
    let mut register_res = client
        .post("/auth/register")
        .header(ContentType::Form)
        .body(format!(
            "username={}&email={}&timezone={timezone}&password={password}&password_confirmation={password}",
            username, email, timezone=timezone, password=password
        ))
        .dispatch();
    assert!(register_res
        .body_string()
        .expect("invalid body response")
        .contains("Registration successful!"));
}

#[cfg(test)]
pub fn login_user(identifier: &str, password: &str, client: &Client) {
    let mut login_res = client
        .post("/auth/login")
        .header(ContentType::Form)
        .body(format!("identifier={}&password={}", identifier, password))
        .dispatch();
    let string = login_res.body_string().expect("invalid body response");
    assert!(string.contains("Logged in"));
    login_res
        .cookies()
        .into_iter()
        .filter(|c| c.name() == LOGIN_COOKIE)
        .next()
        .unwrap();
}

#[cfg(test)]
pub fn logout(client: &Client) {
    assert!(client
        .get("/logout")
        .dispatch()
        .body_string()
        .unwrap()
        .contains("Logged out"));
}
