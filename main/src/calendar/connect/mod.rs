//! Calendar authentication.

use crate::{db::Database, models::calendar::Calendar, utils::default_head};
use diesel::prelude::*;
use malvolio::prelude::*;

/// Authenticated username/password CalDAV integration.
pub mod caldav;
/// Google Calendar integration.
pub mod gcal;
/// *Very* unwise unauthenticated CalDAV integration. Possibly something to remove in the future.
pub mod unauthenticated_caldav;

pub(crate) async fn check_user_does_not_already_have_calendar_connected(
    user_id: i32,
    conn: &Database,
) -> Option<Calendar> {
    use crate::schema::calendar;
    match conn
        .run(move |c| {
            calendar::table
                .filter(calendar::user_id.eq(user_id))
                .first::<Calendar>(c)
        })
        .await
    {
        Ok(t) => Some(t),
        Err(_) => None,
    }
}

pub(crate) fn calendar_already_connected(form: Form) -> Html {
    Html::new().head(default_head("Already connected")).body(
        Body::new()
            .child(H1::new("Error"))
            .child(P::with_text(
                "Error: You have already connected a calendar.",
            ))
            .child(form),
    )
}
