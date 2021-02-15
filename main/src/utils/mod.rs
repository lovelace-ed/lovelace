/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
#[cfg(test)]
use crate::auth::LOGIN_COOKIE;
use crate::calendar::connect::gcal::StateValues;
use malvolio::prelude::{Body, Head, Html, Title, H1, P};
use rocket::figment::{
    util::map,
    value::{Map, Value},
};
use rocket::tokio::sync::RwLock;
use rocket::{fairing::AdHoc, Rocket};
#[cfg(test)]
use rocket::{http::ContentType, local::asynchronous::Client};
use std::collections::HashMap;

pub mod auto_database_error;
pub mod error_messages;
pub mod html_or_redirect;
pub mod json_response;
pub mod permission_error;
pub mod timezones;

pub fn default_head<S>(title: S) -> Head
where
    S: Into<String>,
{
    Head::default().child(Title::new(title.into() + " | Lovelace"))
}

pub fn retrieve_database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://localhost".to_string())
}

pub fn launch() -> Rocket {
    cfg_if! {
        if #[cfg(test)] {
            let db: Map<_, Value> = map! {
                "url" => retrieve_database_url().into(),
                // if the pool size is not one then some of the tests will open multiple connections
                // which means that we run into problems (because we've configured each connection
                // to never commit any data into the database and new connections therefore don't
                // have the same state as the already created ones)
                "pool_size" => 1.into()
            };
        } else {
            let db: Map<_, Value> = map! {
                "url" => retrieve_database_url().into()
            };
        }
    }

    let figment = rocket::Config::figment()
        .merge(("databases", map!["postgres" => db]))
        .merge((
            "secret_key",
            std::env::var("SECRET_KEY")
                .unwrap_or_else(|_| "NNnXxqFeQ/1Sn8lh9MtlIW2uePR4TL/1O5dB2CPkTmg=".to_string()),
        ))
        .merge((
            "port",
            std::env::var("PORT")
                .unwrap_or_else(|_| "5000".to_string())
                .parse::<u16>()
                .expect("invalid $PORT variable supplied"),
        ));
    rocket::custom(figment)
        .manage(StateValues {
            map: RwLock::new(HashMap::new()),
        })
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
                crate::auth::html_logout_user
            ],
        )
        .mount(
            "/api/auth",
            routes![crate::auth::api_login, crate::auth::api_logout,],
        )
        .mount(
            "/auth",
            routes![
                crate::auth::login_page,
                crate::auth::html_login,
                crate::auth::register_page,
                crate::auth::html_register,
                crate::auth::verify_email
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
                crate::class::messages::view_message,
                crate::class::tasks::asynchronous::view_all_async_tasks_in_class,
                crate::class::tasks::asynchronous::create_new_async_task,
                crate::class::tasks::asynchronous::get_create_new_async_task,
                crate::class::tasks::asynchronous::view_specific_asynchronous_task,
                crate::class::tasks::asynchronous::view_edit_task_page,
                crate::class::tasks::asynchronous::apply_edit_task,
                crate::class::tasks::asynchronous::delete_task,
                crate::class::tasks::synchronous::view_all_sync_tasks_in_class,
                crate::class::tasks::synchronous::create_new_sync_task,
                crate::class::tasks::synchronous::get_create_new_sync_task,
                crate::class::tasks::synchronous::view_specific_synchronous_task,
                crate::class::tasks::synchronous::view_edit_task_page,
                crate::class::tasks::synchronous::apply_edit_task,
                crate::class::tasks::synchronous::delete_task
            ],
        )
        .mount(
            "/calendar/gcal",
            routes![
                crate::calendar::connect::gcal::link_calendar,
                crate::calendar::connect::gcal::link_gcal,
                crate::calendar::connect::gcal::gcal_callback
            ],
        )
        .mount(
            "/calendar/unauthenticated_caldav",
            routes![
                crate::calendar::connect::unauthenticated_caldav::link_unauthenticated_caldav,
                crate::calendar::connect::unauthenticated_caldav::view_link_unauthenticated_caldav
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
pub async fn client() -> Client {
    let rocket = launch();
    Client::tracked(rocket)
        .await
        .expect("needs a valid rocket instance")
}

#[cfg(test)]
pub async fn create_user(
    username: &str,
    email: &str,
    timezone: &str,
    password: &str,
    client: &Client,
) {
    let register_res = client
        .post("/auth/register")
        .header(ContentType::Form)
        .body(format!(
            "username={}&email={}&timezone={timezone}&password={password}&password_confirmation={password}",
            username, email, timezone=timezone, password=password
        ))
        .dispatch().await;
    assert!(register_res
        .into_string()
        .await
        .expect("invalid body response")
        .contains("Registration successful!"));
}

#[cfg(test)]
pub async fn login_user(identifier: &str, password: &str, client: &Client) {
    let login_res = client
        .post("/auth/login")
        .header(ContentType::Form)
        .body(format!("identifier={}&password={}", identifier, password))
        .dispatch()
        .await;
    login_res
        .cookies()
        .iter()
        .find(|c| c.name() == LOGIN_COOKIE)
        .unwrap();
    let string = login_res
        .into_string()
        .await
        .expect("invalid body response");
    assert!(string.contains("Logged in"));
}

#[cfg(test)]
pub async fn logout(client: &Client) {
    assert!(client
        .get("/logout")
        .dispatch()
        .await
        .into_string()
        .await
        .unwrap()
        .contains("Logged out"));
}
