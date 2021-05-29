use malvolio::prelude::*;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::json::Json;

use crate::utils::default_head;

use super::LOGIN_COOKIE;

#[get("/logout")]
pub fn html_logout_user(cookies: &CookieJar<'_>) -> Html {
    if cookies.get_private(LOGIN_COOKIE).is_none() {
        return Html::default()
            .head(default_head("Cannot log you out.".to_string()))
            .body(
                Body::default().child(H1::new("You are not logged in, so we cannot log you out.")),
            );
    }
    cookies.remove_private(Cookie::named(LOGIN_COOKIE));
    Html::default()
        .head(default_head("Logged out.".to_string()))
        .body(Body::default().child(H1::new("You are logged out.".to_string())))
}

#[derive(Serialize, Deserialize)]
pub struct LogoutResponse {
    success: bool,
    error: Option<LogoutError>,
}

#[derive(Serialize, Deserialize)]
pub struct LogoutError {
    reason: String,
}

#[get("/logout")]
pub fn api_logout(cookies: &CookieJar<'_>) -> Json<LogoutResponse> {
    Json(if cookies.get_private(LOGIN_COOKIE).is_none() {
        LogoutResponse {
            success: false,
            error: Some(LogoutError {
                reason: "You are not logged in, so you cannot be logged out.".to_string(),
            }),
        }
    } else {
        cookies.remove_private(Cookie::named(LOGIN_COOKIE));
        LogoutResponse {
            success: true,
            error: None,
        }
    })
}
