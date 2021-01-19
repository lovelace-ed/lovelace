use diesel::prelude::*;
use malvolio::{Body, BodyNode, Div, Html, A, H1, H3, P};

use crate::{
    auth::AuthCookie,
    css_names::{LIST, LIST_ITEM},
    db::{Database, DatabaseConnection},
    models::{NewNotification, Notification},
    utils::default_head,
};

fn get_all_notifications<B>(
    user_id: i32,
    conn: &DatabaseConnection,
    custom_element: Option<B>,
) -> Html
where
    B: Into<BodyNode>,
{
    use crate::schema::notifications::dsl as notifications;
    match notifications::notifications
        .filter(notifications::user_id.eq(user_id))
        .filter(notifications::read.eq(false))
        .load::<Notification>(&*conn)
    {
        Ok(data) => {
            Html::default()
                .head(default_head("Notifications".to_string()))
                .body({
                    let mut body = Body::default();
                    if let Some(element) = custom_element {
                        body = body.child(element);
                    }
                    body.child(Div::default().attribute("class", LIST).children(
                        data.into_iter().map(|notification| {
                            Div::default()
                                .attribute("class", LIST_ITEM)
                                .child(H3::new(notification.title))
                                .child(P::with_text(notification.contents))
                                .child(
                                    A::new(format!("/notifications/mark_read/{}", notification.id))
                                        .text("Mark as read"),
                                )
                                .child(
                                    A::new(format!("/notifications/delete/{}", notification.id))
                                        .text("Delete this notification"),
                                )
                        }),
                    ))
                })
        }
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
pub fn list_notifications(auth: AuthCookie, conn: Database) -> Html {
    get_all_notifications::<P>(auth.0, &*conn, None)
}

#[get("/mark_read/<id>")]
pub fn mark_notification_as_read(id: i32, auth: AuthCookie, conn: Database) -> Html {
    use crate::schema::notifications::dsl as notifications;
    match diesel::update(notifications::notifications)
        .set(notifications::read.eq(true))
        .filter(notifications::id.eq(id))
        .filter(notifications::user_id.eq(auth.0))
        .execute(&*conn)
    {
        Ok(_) => get_all_notifications(
            auth.0,
            &*conn,
            Some(P::with_text("Marked that notification as read.")),
        ),
        Err(_) => get_all_notifications(
            auth.0,
            &*conn,
            Some(P::with_text(
                "We encountered a database error trying to mark that notification as read.",
            )),
        ),
    }
}

#[get("/delete/<id>")]
pub fn delete_notification_with_id(id: i32, auth: AuthCookie, conn: Database) -> Html {
    use crate::schema::notifications::dsl as notifications;
    match diesel::delete(
        notifications::notifications
            .filter(notifications::id.eq(id))
            .filter(notifications::user_id.eq(auth.0)),
    )
    .execute(&*conn)
    {
        Ok(_) => get_all_notifications(
            auth.0,
            &*conn,
            Some(P::with_text("Successfully deleted that notification.")),
        ),
        Err(_) => get_all_notifications(
            auth.0,
            &*conn,
            Some(P::with_text("Successfully deleted that notification.")),
        ),
    }
}

#[derive(Debug, Copy, Clone)]
pub enum NotificationPriority {
    Danger,
    Warning,
    Info,
}

impl Into<i16> for NotificationPriority {
    fn into(self) -> i16 {
        match self {
            Self::Danger => 3,
            Self::Warning => 2,
            Self::Info => 1,
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

#[derive(Builder, Clone)]
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
    use rocket::local::Client;

    use crate::{
        db::{Database, TestPgConnection},
        models::{NewNotification, NewUser, Notification},
        utils::{launch, login_user},
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
    #[test]
    fn test_can_view_notifications() {
        let rocket = launch();
        create_dummy_setup(&*Database::get_one(&rocket).unwrap());
        let client = Client::new(rocket).expect("needs a valid rocket instance");
        login_user(EMAIL, PASSWORD, &client);
        let mut notification_list_res = client.get("/notifications/").dispatch();
        let string = notification_list_res
            .body_string()
            .expect("invalid body response");
        assert!(string.contains(NOTIFICATION_1_TITLE));
        assert!(string.contains(NOTIFICATION_1_CONTENTS));
        assert!(string.contains(NOTIFICATION_2_TITLE));
        assert!(string.contains(NOTIFICATION_1_CONTENTS));
    }
    #[test]
    fn test_can_mark_notifications_as_read() {
        let rocket = launch();
        let ids = create_dummy_setup(&*Database::get_one(&rocket).unwrap());
        let client = Client::new(rocket).expect("needs a valid rocket instance");

        login_user(EMAIL, PASSWORD, &client);
        let mut marked_as_read = client
            .get(format!("/notifications/mark_read/{}", ids[0]))
            .dispatch();
        assert!(marked_as_read
            .body_string()
            .expect("invalid body response")
            .contains("notification as read"));
        assert!({
            use crate::schema::notifications::dsl as notifications;
            match notifications::notifications
                .filter(notifications::id.eq(ids[0]))
                .first::<Notification>(&*Database::get_one(client.rocket()).unwrap())
            {
                Ok(t) => t.read,
                Err(_) => false,
            }
        })
    }
    #[test]
    fn test_can_delete_notifications() {
        let rocket = launch();
        let ids = create_dummy_setup(&*Database::get_one(&rocket).unwrap());
        let client = Client::new(rocket).expect("needs a valid rocket instance");
        login_user(EMAIL, PASSWORD, &client);
        let mut deleted = client
            .get(format!("/notifications/delete/{}", ids[0]))
            .dispatch();
        assert!(deleted
            .body_string()
            .expect("invalid body response")
            .contains("deleted that notification"));
        assert!({
            use crate::schema::notifications::dsl as notifications;
            match notifications::notifications
                .filter(notifications::id.eq(ids[0]))
                .first::<Notification>(&*Database::get_one(client.rocket()).unwrap())
            {
                Err(diesel::result::Error::NotFound) => true,
                Ok(_) | Err(_) => false,
            }
        });
    }
}
