//! Google calendar authentication.

use malvolio::prelude::*;
use rocket::tokio::sync::RwLock;
use rocket::{response::Redirect, State};
use std::collections::HashMap;

use diesel::prelude::*;

use crate::{
    auth::AuthCookie,
    catch_database_error,
    db::Database,
    models::calendar::{CalendarType, NewCalendar, NewGoogleCalendar},
    utils::{default_head, html_or_redirect::HtmlOrRedirect},
};

#[derive(Debug, Clone)]
pub struct StateValueEntry {
    pub user_id: i32,
    pub lovelace_calendar_id: String,
}

#[derive(Debug)]
pub struct StateValues {
    pub map: RwLock<HashMap<String, StateValueEntry>>,
}

#[get("/link")]
pub fn link_gcal(_auth: AuthCookie) -> Html {
    Html::new()
        .head(default_head("Connect your Google Calendar.".to_string()))
        .body(
            Body::default()
                .child(H1::new("Connect your Google Calendar"))
                .child(
                    Form::new()
                        .attribute(Method::Post)
                        .child(
                            Input::new()
                                .attribute(Type::Text)
                                .attribute(Placeholder::new("The calendar URL to update")),
                        )
                        .child(
                            Input::new()
                                .attribute(Type::Submit)
                                .attribute(Value::new("Connect your calendar.")),
                        ),
                ),
        )
}

#[derive(FromForm, Debug)]
pub struct LinkCalendarForm {
    url: String,
}

#[post("/link", data = "<form>")]
pub async fn link_calendar(
    auth: AuthCookie,
    oauth_state_values: &State<StateValues>,
    form: rocket::form::Form<LinkCalendarForm>,
) -> HtmlOrRedirect {
    let uuid = uuid::Uuid::new_v4().to_string();

    oauth_state_values.map.write().await.insert(
        uuid.clone(),
        StateValueEntry {
            user_id: auth.0,
            lovelace_calendar_id: form.url.clone(),
        },
    );
    let res = format!(
        "{}?state={}&?scope=https%3A//www.googleapis.com/auth/calendar?redirect_uri={}?client_id={}",
        std::env::var("OAUTH_TEST_SERVER")
            .unwrap_or_else(|_| "https://accounts.google.com/o/oauth2/v2/auth".to_string()),
        uuid,
        format!(
            "{}/calendar/auth/gcal/callback",
            std::env::var("HOSTNAME")
                .expect("the `HOSTNAME` environment variable has not been set")
        ),
        std::env::var("CLIENT_ID").expect("the `CLIENT_ID` environment variable has not been set")
    );
    HtmlOrRedirect::Redirect(Redirect::to(res))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccessTokenResponse {
    access_token: String,
    expires_in: Option<i32>,
    token_type: Option<String>,
    scope: Option<String>,
    refresh_token: String,
}

#[get("/callback?<code>&<error>&<state>")]
pub async fn gcal_callback(
    code: Option<String>,
    error: Option<String>,
    state: Option<String>,
    oauth_state_values: &State<StateValues>,
    conn: Database,
) -> Html {
    use crate::schema::calendar;
    use crate::schema::google_calendar;
    if error.is_some() {
        return Html::new();
    }

    let lock = oauth_state_values.map.read().await;
    if let Some(code) = code {
        if let Some(state) = state {
            if let Some(entry) = lock.get(&state) {
                let entry = entry.clone();
                // we drop the lock here because we want to read from it later.
                std::mem::drop(lock);
                let access_token_response: AccessTokenResponse = ureq::post(
                    &std::env::var("TOKEN_URL")
                        .unwrap_or_else(|_| "https://oauth2.googleapis.com/token".to_string()),
                )
                .set("Content-Type", "application/x-www-form-urlencoded")
                .send_string(&format!(
                    "code={}?
                        client_id={}?
                        client_secret={}?
                        redirect_uri={}?
                        grant_type=authorization_code",
                    code,
                    std::env::var("CLIENT_ID").unwrap(),
                    std::env::var("CLIENT_SECRET").unwrap(),
                    std::env::var("REDIRECT_URI").unwrap()
                ))
                .unwrap()
                .into_json()
                .unwrap();
                let move_entry_user_id = entry.user_id;
                let calendar_id = catch_database_error!(
                    conn.run(move |c| diesel::insert_into(calendar::table)
                        .values(NewCalendar {
                            calendar_type: CalendarType::GoogleCalendar.into(),
                            user_id: move_entry_user_id,
                        })
                        .returning(calendar::id)
                        .get_result::<i32>(c))
                        .await
                );
                cfg_if! {
                    if #[cfg(test)] {
                        let to_update = format!(
                            "http://localhost:8080/user/calendars/{}",
                            entry.lovelace_calendar_id
                        );
                    } else {
                        let to_update = format!(
                            "https://apidata.googleusercontent.com/caldav/v2/{}/events",
                            entry.lovelace_calendar_id
                        );
                    }
                };
                catch_database_error!(
                    conn.run(move |c| diesel::insert_into(google_calendar::table)
                        .values(NewGoogleCalendar {
                            refresh_token: &access_token_response.refresh_token,
                            access_token: &access_token_response.access_token,
                            calendar_id,
                            lovelace_calendar_id: &to_update
                        })
                        .execute(c))
                        .await
                );
                oauth_state_values.map.write().await.remove(&state);
                Html::new()
                    .head(default_head("Head".to_string()))
                    .body(Body::new().child(H1::new("Connected your calendar")))
            } else {
                Html::new()
                    .head(default_head("Error".to_string()))
                    .body(Body::new().child(P::with_text(
                        "Not processing supplied code without a valid `state` query string.",
                    )))
            }
        } else {
            Html::new()
                .head(default_head("Error".to_string()))
                .body(Body::new().child(P::with_text(
                    "Not processing supplied code without a valid `state` query string.",
                )))
        }
    } else {
        Html::new()
            .head(default_head("Error".to_string()))
            .body(Body::new().child(P::with_text("Did not get a valid code to process.")))
    }
}
