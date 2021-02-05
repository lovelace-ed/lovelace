use super::{calendar_already_connected, check_user_does_not_already_have_calendar_connected};
use crate::schema::{caldav_unauthenticated, calendar};
use crate::{
    auth::AuthCookie,
    catch_database_error,
    db::Database,
    models::calendar::{CalendarType, NewCalDAVUnauthenticated, NewCalendar},
    utils::default_head,
};
use diesel::prelude::*;
use malvolio::prelude::*;

fn caldav_form() -> Form {
    Form::new()
        .child(
            Input::new()
                .attribute(Name::new("url"))
                .attribute(Placeholder::new("The URL of the calendar")),
        )
        .child(
            Input::new()
                .attribute(Type::Submit)
                .attribute(Value::new("Submit")),
        )
}

#[get("/link")]
pub async fn view_link_unauthenticated_caldav(auth: AuthCookie, conn: Database) -> Html {
    if check_user_does_not_already_have_calendar_connected(auth.0, &conn)
        .await
        .is_some()
    {
        return calendar_already_connected(caldav_form());
    }
    Html::new()
        .head(default_head("Connect a callendar".to_string()))
        .body(
            Body::new()
                .child(H1::new("Connect a calendar."))
                .child(caldav_form()),
        )
}

#[derive(FromForm, Debug)]
pub struct Unauthenticated {
    url: String,
}

#[post("/link", data = "<form>")]
pub async fn link_unauthenticated_caldav(
    form: rocket::request::Form<Unauthenticated>,
    auth: AuthCookie,
    conn: Database,
) -> Html {
    if check_user_does_not_already_have_calendar_connected(auth.0, &conn)
        .await
        .is_some()
    {
        return calendar_already_connected(caldav_form());
    }
    let calendar_id = catch_database_error!({
        let user_id = auth.0;
        conn.run(move |c| {
            diesel::insert_into(calendar::table)
                .values(NewCalendar {
                    calendar_type: CalendarType::CalDAVUnauthenticated.into(),
                    user_id,
                })
                .returning(calendar::id)
                .get_result(c)
        })
        .await
    });
    catch_database_error!(
        conn.run(move |c| diesel::insert_into(caldav_unauthenticated::table)
            .values(NewCalDAVUnauthenticated {
                calendar_id,
                url: &form.url
            })
            .execute(c))
            .await
    );
    Html::new()
        .head(default_head("Added that calendar".to_string()))
        .body(
            Body::new()
                .child(H1::new("Created that calendar"))
                .child(P::with_text("That calendar has been added.")),
        )
}
