use bcrypt::verify;
use diesel::prelude::*;
use malvolio::prelude::*;
use mercutio::Apply;
use portia::form::{FormStyle, FormSubmitInputStyle, FormTextInputStyle};
use rocket::http::{Cookie, CookieJar};
use rocket_contrib::json::Json;
use thiserror::Error as ThisError;

use crate::{db::Database, models::User, utils::default_head};

use super::LOGIN_COOKIE;

fn login_form() -> malvolio::prelude::Form {
    malvolio::prelude::Form::new()
        .apply(FormStyle)
        .attribute(Method::Post)
        .child(
            Input::default()
                .apply(FormTextInputStyle)
                .attribute(Type::Text)
                .attribute(Placeholder::new("Username"))
                .attribute(Name::new("identifier")),
        )
        .child(Br)
        .child(
            Input::default()
                .apply(FormTextInputStyle)
                .attribute(Type::Password)
                .attribute(Placeholder::new("Password"))
                .attribute(Name::new("password")),
        )
        .child(Br)
        .child(
            Input::default()
                .apply(FormSubmitInputStyle)
                .attribute(Type::Submit)
                .attribute(Value::new("Login!")),
        )
}

#[get("/login")]
pub fn login_page() -> Html {
    Html::default()
        .head(default_head("Login".to_string()))
        .body(Body::default().child(H1::new("Login")).child(login_form()))
}

#[derive(ThisError, Debug)]
pub enum LoginError {
    #[error("user not found")]
    UserNotFound,
    #[error("password not valid")]
    PasswordNotValid,
    #[error("database error")]
    DatabaseError,
}

async fn login_base(
    cookies: &CookieJar<'_>,
    data: &LoginData,
    conn: Database,
) -> Result<User, LoginError> {
    use crate::schema::users;
    let closure_data = data.clone();
    match conn
        .run(move |c| {
            users::table
                .filter(users::username.eq(&closure_data.identifier))
                .or_filter(users::email.eq(&closure_data.identifier))
                .first::<User>(c)
        })
        .await
    {
        Ok(user) => {
            if verify(&data.password, &user.password)
                .map_err(|e| error!("{:#?}", e))
                .unwrap_or(false)
            {
                cookies.add_private(Cookie::new(LOGIN_COOKIE, user.id.to_string()));
                Ok(user)
            } else {
                Err(LoginError::PasswordNotValid)
            }
        }
        Err(error) => match error {
            diesel::result::Error::NotFound => Err(LoginError::UserNotFound),
            _ => Err(LoginError::DatabaseError),
        },
    }
}

#[derive(FromForm, Debug, Clone, Serialize, Deserialize)]
pub struct LoginData {
    identifier: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    success: bool,
    data: Option<User>,
    error: Option<LoginErrorExplanation>,
}

#[derive(Serialize)]
pub struct LoginErrorExplanation {
    reason: String,
}

#[post("/login", data = "<data>")]
pub async fn api_login(
    cookies: &CookieJar<'_>,
    data: Json<LoginData>,
    conn: Database,
) -> Json<LoginResponse> {
    Json(match login_base(cookies, &data, conn).await {
        Ok(user) => LoginResponse {
            success: true,
            data: Some(user),
            error: None,
        },
        Err(error) => match error {
            LoginError::UserNotFound => LoginResponse {
                success: false,
                data: None,
                error: Some(LoginErrorExplanation {
                    reason: "A user with that username or email could not be found.".to_string(),
                }),
            },
            LoginError::PasswordNotValid => LoginResponse {
                success: false,
                data: None,
                error: Some(LoginErrorExplanation {
                    reason: "That password is not correct.".to_string(),
                }),
            },
            LoginError::DatabaseError => LoginResponse {
                success: false,
                data: None,
                error: Some(LoginErrorExplanation {
                    reason: "There was an internal database error logging in.".to_string(),
                }),
            },
        },
    })
}

#[post("/login", data = "<data>")]
pub async fn html_login(
    cookies: &CookieJar<'_>,
    data: rocket::request::Form<LoginData>,
    conn: Database,
) -> Html {
    match login_base(cookies, &data, conn).await {
        Ok(_) => Html::default()
            .head(default_head("Logged in".to_string()))
            .body(
                Body::default()
                    .child(H1::new("Logged in!"))
                    .child(P::with_text("You are now logged in.")),
            ),
        Err(e) => match e {
            LoginError::UserNotFound => Html::default()
                .status(404)
                .head(default_head("Not found".to_string()))
                .body(
                    Body::default()
                        .child(H1::new("Login error"))
                        .child(P::with_text(
                            "We searched everywhere (in our database) but we couldn't \
                                    find a user with that username or email.",
                        ))
                        .child(login_form()),
                ),
            LoginError::PasswordNotValid => Html::default()
                .status(400)
                .head(default_head("Error".to_string()))
                .body(
                    Body::default()
                        .child(H1::new("Invalid password"))
                        .child(P::with_text("The password you've supplied isn't correct."))
                        .child(login_form()),
                ),
            LoginError::DatabaseError => Html::default()
                .status(500)
                .head(default_head("Unknown error".to_string()))
                .body(
                    Body::default()
                        .child(H1::new("Database error"))
                        .child(P::with_text(
                            "Something's up on our end. We're working to fix it as fast as we can!",
                        ))
                        .child(login_form()),
                ),
        },
    }
}
