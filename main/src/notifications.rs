/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

use diesel::prelude::*;
use malvolio::prelude::{Body, BodyNode, Div, Href, Html, A, H1, H3, P};
use rocket_contrib::json::Json;

use crate::{
    auth::AuthCookie,
    css_names::{LIST, LIST_ITEM},
    db::{Database, DatabaseConnection},
    models::{NewNotification, Notification},
    utils::{default_head, json_response::ApiResponse},
};

async fn retrieve_notifications(
    user_id: i32,
    conn: &Database,
) -> Result<Vec<Notification>, diesel::result::Error> {
    use crate::schema::notifications::dsl as notifications;
    conn.run(move |c| {
        notifications::notifications
            .filter(notifications::user_id.eq(user_id))
            .filter(notifications::read.eq(false))
            .load::<Notification>(c)
    })
    .await
}

fn render_notifications<B>(
    data: Result<Vec<Notification>, diesel::result::Error>,
    custom_element: Option<B>,
) -> Html
where
    B: Into<BodyNode>,
{
    match data {
        Ok(data) => Html::default()
            .head(default_head("Notifications".to_string()))
            .body({
                let mut body = Body::default();
                if let Some(element) = custom_element {
                    body = body.child(element);
                }
                body.child(
                    Div::new()
                        .attribute(malvolio::prelude::Class::from(LIST))
                        .children(data.into_iter().map(|notification| {
                            Div::new()
                                .attribute(malvolio::prelude::Class::from(LIST_ITEM))
                                .child(H3::new(notification.title))
                                .child(P::with_text(notification.contents))
                                .child(
                                    A::default()
                                        .attribute(Href::new(format!(
                                            "/notifications/mark_read/{}",
                                            notification.id
                                        )))
                                        .text("Mark as read"),
                                )
                                .child(
                                    A::default()
                                        .attribute(Href::new(format!(
                                            "/notifications/delete/{}",
                                            notification.id
                                        )))
                                        .text("Delete this notification"),
                                )
                        })),
                )
            }),
        Err(e) => {
            error!("Error retrieving notifications: {:?}", e);
            Html::default()
                .head(default_head("Notifications".to_string()))
                .body(
                    Body::default()
                        .child(H1::new("Error retrieving notifications."))
                        .child(P::with_text(
                            "We encountered a database error trying to retrieve your
                    notifications from the database.",
                        )),
                )
        }
    }
}

#[get("/")]
pub async fn api_list_notifications(
    auth: AuthCookie,
    conn: Database,
) -> Json<ApiResponse<Vec<Notification>>> {
    Json(match retrieve_notifications(auth.0, &conn).await {
        Ok(notifications) => ApiResponse::new_ok(notifications),
        Err(_) => {
            ApiResponse::new_err("Encountered a database error trying to fulfil that request.")
        }
    })
}

#[get("/")]
pub async fn list_notifications(auth: AuthCookie, conn: Database) -> Html {
    let data = retrieve_notifications(auth.0, &conn).await;
    render_notifications::<P>(data, None)
}

async fn mark_read(
    id: i32,
    auth: AuthCookie,
    conn: &Database,
) -> Result<Notification, diesel::result::Error> {
    use crate::schema::notifications::dsl as notifications;

    conn.run(move |c| {
        diesel::update(notifications::notifications)
            .set(notifications::read.eq(true))
            .filter(notifications::id.eq(id))
            .filter(notifications::user_id.eq(auth.0))
            .returning(crate::schema::notifications::all_columns)
            .get_result(c)
    })
    .await
}

#[get("/mark_read/<id>")]
pub async fn api_mark_notification_as_read(
    id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Json<ApiResponse<Notification>> {
    Json(match mark_read(id, auth, &conn).await {
        Ok(notification) => ApiResponse::new_ok(notification),
        Err(_) => ApiResponse::new_err(
            "Encountered a database error when trying to update that item in the database.",
        ),
    })
}

#[get("/mark_read/<id>")]
pub async fn mark_notification_as_read(id: i32, auth: AuthCookie, conn: Database) -> Html {
    match mark_read(id, auth, &conn).await {
        Ok(_) => {
            let data = retrieve_notifications(auth.0, &conn).await;
            render_notifications(
                data,
                Some(P::with_text("Marked that notification as read.")),
            )
        }
        Err(_) => {
            let data = retrieve_notifications(auth.0, &conn).await;
            render_notifications(
                data,
                Some(P::with_text(
                    "We encountered a database error trying to mark that notification as read.",
                )),
            )
        }
    }
}

async fn delete_notification(
    id: i32,
    auth: AuthCookie,
    conn: &Database,
) -> Result<(), diesel::result::Error> {
    use crate::schema::notifications::dsl as notifications;
    conn.run(move |c| {
        diesel::delete(
            notifications::notifications
                .filter(notifications::id.eq(id))
                .filter(notifications::user_id.eq(auth.0)),
        )
        .execute(c)
    })
    .await
    .map(|_| ())
}

#[get("/delete/<id>")]
pub async fn api_delete_notification_with_id(
    id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Json<ApiResponse<()>> {
    Json(match delete_notification(id, auth, &conn).await {
        Ok(_) => ApiResponse::new_ok(()),
        Err(_) => ApiResponse::new_ok(()),
    })
}

#[get("/delete/<id>")]
pub async fn html_delete_notification_with_id(id: i32, auth: AuthCookie, conn: Database) -> Html {
    match delete_notification(id, auth, &conn).await {
        Ok(_) => {
            let data = retrieve_notifications(auth.0, &conn).await;
            render_notifications(
                data,
                Some(P::with_text("Successfully deleted that notification.")),
            )
        }
        Err(_) => {
            let data = retrieve_notifications(auth.0, &conn).await;
            render_notifications(
                data,
                Some(P::with_text("Successfully deleted that notification.")),
            )
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum NotificationPriority {
    Danger,
    Warning,
    Info,
}

impl From<NotificationPriority> for i16 {
    fn from(from: NotificationPriority) -> Self {
        match from {
            NotificationPriority::Danger => 3,
            NotificationPriority::Warning => 2,
            NotificationPriority::Info => 1,
        }
    }
}

impl From<i16> for NotificationPriority {
    /// Converts a row in the database to an `i32`, `panic`-ing if the database contains invalid
    /// data. To ensure that this never happens, make sure to never insert an integer value directly
    /// into the `notifications.priority` column – instead use `NotificationPriority` (which
    /// implements `Into<i32>` for this purpose).
    fn from(number: i16) -> Self {
        match number {
            1 => Self::Info,
            2 => Self::Warning,
            3 => Self::Danger,
            number => {
                error!("Invalid number in database: {}", number);
                panic!()
            }
        }
    }
}

#[derive(Builder, Clone, Debug)]
/// A struct used to send notifications to a user. This struct can be created with the automagically
/// generated `NotifyBuilder` and dispatched with the `create` method.
pub struct Notify<'a> {
    intended_for: i32,
    title: &'a str,
    message: &'a str,
    priority: NotificationPriority,
}

impl<'a> Notify<'a> {
    /// Add the current struct to the database.
    #[allow(unused)]
    pub fn create(&self, conn: &DatabaseConnection) -> Result<(), diesel::result::Error> {
        use crate::schema::notifications::dsl as notifications;
        diesel::insert_into(notifications::notifications)
            .values(NewNotification::new(
                self.title,
                self.message,
                chrono::Utc::now().naive_utc(),
                self.priority,
                self.intended_for,
                false,
            ))
            .execute(conn)
            .map(drop)
    }
}

#[cfg(test)]
mod test {
    use bcrypt::DEFAULT_COST;
    use diesel::prelude::*;

    use crate::{
        db::{Database, TestPgConnection},
        models::{NewNotification, NewUser, Notification},
        utils::{client, login_user},
    };

    use super::NotificationPriority;

    const USERNAME: &str = "some-username";
    const EMAIL: &str = "email@example.com";
    const PASSWORD: &str = "passw0rdWhichPass3sCriteria";
    const TIMEZONE: &str = "Africa/Abidjan";

    const NOTIFICATION_1_TITLE: &str = "sometitleinmessage1only";
    const NOTIFICATION_1_CONTENTS: &str = "message1contentswithsp3cialcharact3rs";
    const NOTIFICATION_1_PRIORITY: NotificationPriority = NotificationPriority::Info;

    const NOTIFICATION_2_TITLE: &str = "somemessage2title";
    const NOTIFICATION_2_CONTENTS: &str = "message2contentswith3xtrasp3cialcharact3rs";
    const NOTIFICATION_2_PRIORITY: NotificationPriority = NotificationPriority::Info;

    fn create_dummy_setup(conn: &TestPgConnection) -> Vec<i32> {
        use crate::schema::notifications::dsl as notifications;
        use crate::schema::users::dsl as users;
        let user_id: i32 = diesel::insert_into(users::users)
            .values(&NewUser::new(
                USERNAME,
                EMAIL,
                bcrypt::hash(PASSWORD, DEFAULT_COST).unwrap().as_ref(),
                chrono::Utc::now().naive_utc(),
                TIMEZONE,
            ))
            .returning(users::id)
            .get_result(conn)
            .expect("failed to create users");
        diesel::insert_into(notifications::notifications)
            .values(&vec![
                NewNotification::new(
                    NOTIFICATION_1_TITLE,
                    NOTIFICATION_1_CONTENTS,
                    chrono::Utc::now().naive_utc(),
                    NOTIFICATION_1_PRIORITY,
                    user_id,
                    false,
                ),
                NewNotification::new(
                    NOTIFICATION_2_TITLE,
                    NOTIFICATION_2_CONTENTS,
                    chrono::Utc::now().naive_utc(),
                    NOTIFICATION_2_PRIORITY,
                    user_id,
                    false,
                ),
            ])
            .returning(notifications::id)
            .get_results(conn)
            .expect("failed to add notifications")
    }
    #[rocket::async_test]
    async fn test_can_view_notifications() {
        let client = client().await;
        Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| create_dummy_setup(c))
            .await;
        login_user(EMAIL, PASSWORD, &client).await;
        let notification_list_res = client.get("/notifications/").dispatch().await;
        let string = notification_list_res
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains(NOTIFICATION_1_TITLE));
        assert!(string.contains(NOTIFICATION_1_CONTENTS));
        assert!(string.contains(NOTIFICATION_2_TITLE));
        assert!(string.contains(NOTIFICATION_1_CONTENTS));
    }
    #[rocket::async_test]
    async fn test_can_mark_notifications_as_read() {
        let client = client().await;
        let ids = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| create_dummy_setup(c))
            .await;

        login_user(EMAIL, PASSWORD, &client).await;

        let marked_as_read = client
            .get(format!("/notifications/mark_read/{}", ids[0]))
            .dispatch()
            .await;
        assert!(marked_as_read
            .into_string()
            .await
            .expect("invalid body response")
            .contains("notification as read"));
        assert!({
            use crate::schema::notifications::dsl as notifications;
            match Database::get_one(client.rocket())
                .await
                .unwrap()
                .run(move |c| {
                    notifications::notifications
                        .filter(notifications::id.eq(ids[0]))
                        .first::<Notification>(c)
                })
                .await
            {
                Ok(t) => t.read,
                Err(_) => false,
            }
        })
    }
    #[rocket::async_test]
    async fn test_can_delete_notifications() {
        let client = client().await;
        let ids = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(move |c| create_dummy_setup(c))
            .await;
        login_user(EMAIL, PASSWORD, &client).await;
        let deleted = client
            .get(format!("/notifications/delete/{}", ids[0]))
            .dispatch()
            .await;
        assert!(deleted
            .into_string()
            .await
            .expect("invalid body response")
            .contains("deleted that notification"));
        assert!({
            use crate::schema::notifications::dsl as notifications;
            match Database::get_one(client.rocket())
                .await
                .unwrap()
                .run(move |c| {
                    notifications::notifications
                        .filter(notifications::id.eq(ids[0]))
                        .first::<Notification>(c)
                })
                .await
            {
                Err(diesel::result::Error::NotFound) => true,
                Ok(_) | Err(_) => false,
            }
        });
    }
}
