/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
#[cfg(test)]
use crate::auth::LOGIN_COOKIE;
use crate::calendar::connect::gcal::StateValues;
#[cfg(test)]
use crate::db::Database;
#[cfg(test)]
use crate::models::NewUser;
#[cfg(test)]
use crate::schema::users;
#[cfg(test)]
use chrono::Utc;
use malvolio::prelude::{Body, Head, Html, Title, H1, P};
use rocket::tokio::sync::RwLock;
use rocket::{fairing::AdHoc, Rocket};
use rocket::{
    figment::{
        util::map,
        value::{Map, Value},
        Figment,
    },
    Build,
};
#[cfg(test)]
use rocket::{http::ContentType, local::asynchronous::Client};
use std::{collections::HashMap, net::IpAddr};

pub mod auto_database_error;
pub mod error;
pub mod error_messages;
pub mod form;
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

pub fn launch() -> Rocket<Build> {
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
            let db: Map<_, Value> = if let Some(pool_size) = std::env::var("POOL_SIZE")
                .ok().map(|item| item.parse::<u32>().ok()).flatten() {
                map! {
                    "url" => retrieve_database_url().into(),
                    "pool_size" => pool_size.into()
                }
            } else {
                map! {
                    "url" => retrieve_database_url().into()
                }
            };
        }
    }

    let figment = if let Ok(secret) = std::env::var("SECRET_KEY") {
        Figment::from(rocket::Config::default())
            .merge((
                "port",
                std::env::var("PORT")
                    .unwrap_or_else(|_| "8000".to_string())
                    .as_str()
                    .parse::<u16>()
                    .expect("invalid port supplied"),
            ))
            .merge((
                "address",
                std::env::var("HOST")
                    .unwrap_or_else(|_| "127.0.0.1".to_string())
                    .as_str()
                    .parse::<IpAddr>()
                    .expect("invalid host supplied"),
            ))
            .merge(("secret_key", secret.as_str()))
            .merge(("databases", map!["postgres" => db]))
    } else {
        Figment::from(rocket::Config::default()).merge(("databases", map!["postgres" => db]))
    };
    rocket::custom(figment)
        .manage(StateValues {
            map: RwLock::new(HashMap::new()),
        })
        .attach(crate::db::Database::fairing())
        .attach(AdHoc::try_on_ignite(
            "Database Migrations",
            crate::db::run_migrations,
        ))
        .mount(
            "/api",
            routes![
                crate::class::api_create_class,
                crate::class::api_delete_class,
                crate::class::api_join_class,
                crate::class::api_view_class_members_page,
                crate::class::api_view_all_classes,
                crate::class::api_invite_teacher,
                crate::class::api_view_class_overview,
            ],
        )
        .mount(
            "/",
            routes![
                crate::home::home,
                crate::class::html_create_class,
                crate::class::create_class_page,
                crate::class::html_join_class,
                crate::class::html_view_all_classes,
                crate::class::html_view_class_overview,
                crate::class::get_class_settings,
                crate::class::html_view_class_members_page,
                crate::class::invite_teacher_page,
                crate::class::html_invite_teacher,
                crate::class::delete_class_page,
                crate::class::html_delete_class,
                crate::auth::html_logout_user
            ],
        )
        .mount("/api/dashboard", routes![crate::dashboard::api_dashboard])
        .mount("/dashboard", routes![crate::dashboard::html_dashboard])
        .mount(
            "/api/auth",
            routes![crate::auth::api_login, crate::auth::api_logout, crate::auth::api_register],
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
            "/api/institution",
            routes![
                crate::institution::register::api_register_new_institution,
                crate::institution::delete::api_delete_institution,
                crate::institution::configure::api_configure_institution,
                crate::institution::class::create::api_create_institution_class
            ],
        )
        .mount(
            "/institution",
            routes![
                crate::institution::register::register_new_institution_page,
                crate::institution::register::html_register_new_institution,
                crate::institution::delete::delete_institution_page,
                crate::institution::delete::html_delete_institution,
                crate::institution::configure::configure_institution_page,
                crate::institution::configure::html_configure_institution,
                crate::institution::class::create::pick_which_institution_to_create_class_as_part_of,
                crate::institution::class::create::html_create_institution_class,
                crate::institution::class::create::create_institution_class_page
            ],
        )
        .mount(
            "/notifications",
            routes![
                crate::notifications::list_notifications,
                crate::notifications::mark_notification_as_read,
                crate::notifications::html_delete_notification_with_id
            ],
        )
        .mount(
            "/api/notifications",
            routes![
                crate::notifications::api_list_notifications,
                crate::notifications::api_mark_notification_as_read,
                crate::notifications::api_delete_notification_with_id
            ]
        )
        .mount(
            "/api/class",
            routes![
                crate::class::messages::api_list_all_messages,
                crate::class::messages::api_apply_message_edit,
                crate::class::messages::api_view_message,
                crate::class::messages::api_reply_to_teacher_message,
                crate::class::messages::api_apply_create_new_class_message,
                crate::class::messages::create_new_class_message_page,
                crate::class::messages::api_apply_message_reply_edit,
                crate::class::tasks::asynchronous::api_create_new_async_task,
                crate::class::tasks::asynchronous::api_apply_edit_task,
                crate::class::tasks::asynchronous::api_view_specific_asynchronous_task,
                crate::class::tasks::asynchronous::api_delete_task,
                crate::class::tasks::asynchronous::api_view_all_async_tasks_in_class,
                crate::class::tasks::synchronous::api_create_new_async_task,
                crate::class::tasks::synchronous::api_delete_task,
                crate::class::tasks::synchronous::api_apply_edit_task,
                crate::class::tasks::synchronous::api_view_specific_synchronous_task,
                crate::class::tasks::synchronous::api_view_all_sync_tasks_in_class
            ],
        )
        .mount(
            "/class",
            routes![
                crate::class::messages::html_list_all_messages,
                crate::class::messages::html_apply_create_new_class_message,
                crate::class::messages::html_reply_to_teacher_message,
                crate::class::messages::edit_message_page,
                crate::class::messages::html_apply_message_edit,
                crate::class::messages::edit_message_reply,
                crate::class::messages::html_apply_message_reply_edit,
                crate::class::messages::view_message,
                crate::class::tasks::asynchronous::html_view_all_async_tasks_in_class,
                crate::class::tasks::asynchronous::html_create_new_async_task,
                crate::class::tasks::asynchronous::get_create_new_async_task,
                crate::class::tasks::asynchronous::html_view_specific_asynchronous_task,
                crate::class::tasks::asynchronous::view_edit_task_page,
                crate::class::tasks::asynchronous::html_apply_edit_task,
                crate::class::tasks::asynchronous::html_delete_task,
                crate::class::tasks::synchronous::html_view_all_sync_tasks_in_class,
                crate::class::tasks::synchronous::html_create_new_sync_task,
                crate::class::tasks::synchronous::get_create_new_sync_task,
                crate::class::tasks::synchronous::html_view_specific_synchronous_task,
                crate::class::tasks::synchronous::view_edit_task_page,
                crate::class::tasks::synchronous::html_apply_edit_task,
                crate::class::tasks::synchronous::html_delete_task
            ],
        )
        .mount(
            "/calendar/caldav",
            routes![
                crate::calendar::connect::caldav::link_caldav_page,
                crate::calendar::connect::caldav::connect_caldav_calendar
            ]
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
    username: &'static str,
    email: &'static str,
    timezone: &'static str,
    password: &'static str,
    client: &Client,
) {
    use diesel::prelude::*;
    Database::get_one(client.rocket())
        .await
        .unwrap()
        .run(move |c| {
            diesel::insert_into(users::table)
                .values(NewUser {
                    username,
                    email,
                    password: &bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap(),
                    created: Utc::now().naive_utc(),
                    email_verified: true,
                    timezone,
                })
                .execute(c)
        })
        .await
        .expect("failed to register");
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
