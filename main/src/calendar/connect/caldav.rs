use std::ops::Add;

use super::{calendar_already_connected, check_user_does_not_already_have_calendar_connected};
use crate::{
    auth::AuthCookie,
    catch_database_error,
    db::Database,
    models::calendar::{CalendarType, NewCalDav, NewCalendar},
    utils::{default_head, error_messages::database_error},
};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use malvolio::prelude::*;
use prospero::client::DavClient;
use rocket::FromForm;

fn caldav_form() -> Form {
    Form::new()
        .child(
            Input::new()
                .attribute(Name::new("username"))
                .attribute(Placeholder::new("Username for the CalDAV server."))
                .attribute(Type::Text),
        )
        .child(
            Input::new()
                .attribute(Name::new("password"))
                .attribute(Placeholder::new("Password for the CalDAV server."))
                .attribute(Type::Text),
        )
        .child(
            Input::new()
                .attribute(Name::new("url"))
                .attribute(Placeholder::new("URL for the CalDAV server."))
                .attribute(Type::Text),
        )
        .child(
            Input::new()
                .attribute(Type::Submit)
                .attribute(Value::new("Connect calendar")),
        )
}

#[get("/link")]
pub async fn link_caldav_page(auth: AuthCookie, conn: Database) -> Html {
    if check_user_does_not_already_have_calendar_connected(auth.0, &conn)
        .await
        .is_some()
    {
        return calendar_already_connected(caldav_form());
    }
    Html::new()
        .head(default_head("Link a CalDAV client".to_string()))
        .body(
            Body::new()
                .child(H1::new("Connect a CalDAV compatible calendar"))
                .child(caldav_form()),
        )
}

#[derive(FromForm, Debug, Clone)]
pub struct CaldavCalendarForm {
    username: String,
    password: String,
    url: String,
}

#[post("/link", data = "<form>")]
pub async fn connect_caldav_calendar(
    conn: Database,
    form: rocket::request::Form<CaldavCalendarForm>,
    auth: AuthCookie,
) -> Html {
    use crate::schema::caldav;
    use crate::schema::calendar;
    if check_user_does_not_already_have_calendar_connected(auth.0, &conn)
        .await
        .is_some()
    {
        return calendar_already_connected(caldav_form());
    }
    let client = DavClient::new_username_password(&form.username, &form.password, &form.url);

    match client
        .calendar()
        .date_search(Utc::now(), Utc::now().add(Duration::days(14)))
        .await
    {
        Ok(_) => match conn
            .run(move |c| {
                diesel::insert_into(calendar::table)
                    .values(NewCalendar {
                        calendar_type: CalendarType::CalDav.into(),
                        user_id: auth.0,
                    })
                    .returning(calendar::id)
                    .get_result::<i32>(c)
            })
            .await
        {
            Ok(res) => {
                catch_database_error!(
                    conn.run(move |c| diesel::insert_into(caldav::table)
                        .values(NewCalDav {
                            calendar_id: res,
                            username: &form.username,
                            password: &form.password,
                            url: &form.url
                        })
                        .execute(c))
                        .await
                );
                Html::new().head(default_head("Success".to_string())).body(
                    Body::new()
                        .child(H1::new("Added that calendar."))
                        .child(P::with_text(
                            "We will start scheduling things into it soon.",
                        )),
                )
            }
            Err(_) => database_error(),
        },
        Err(_) => Html::new().head(default_head("Error".to_string())).body(
            Body::new()
                .child(H1::new("Error"))
                .child(P::with_text(
                    "Error: we tried to contact the provided server, but the response was invalid.",
                ))
                .child(caldav_form()),
        ),
    }
}

#[cfg(test)]
#[cfg(feature = "caldav_server")]
mod test_connect_caldav {
    use crate::{
        db::{Database, DatabaseConnection},
        models::NewUser,
        utils::client,
    };
    use chrono::Utc;
    use diesel::prelude::*;

    const USERNAME: &str = "someuser";
    const EMAIL: &str = "someuser@example.com";
    const PASSWORD: &str = "arand0mishpassw0rd";
    const TIMEZONE: &str = "Africa/Abidjan";

    fn setup_env(conn: &DatabaseConnection) {
        use crate::schema::users;
        diesel::insert_into(users::table)
            .values(NewUser {
                username: USERNAME,
                email: EMAIL,
                password: PASSWORD,
                created: Utc::now().naive_utc(),
                email_verified: true,
                timezone: TIMEZONE,
            })
            .execute(conn)
            .expect("failed to create user");
    }
    #[rocket::async_test]
    async fn test_can_connect_caldav() {
        let client = client().await;
        Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(|c| setup_env(c))
            .await;
    }
}
