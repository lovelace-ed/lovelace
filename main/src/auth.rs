use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use diesel::insert_into;
use diesel::prelude::*;
use malvolio::{Body, Br, Html, Input, H1, P};
use regex::Regex;
use rocket::{
    http::{Cookie, Cookies, Status},
    request::{Form, FromRequest},
};
use thiserror::Error as ThisError;

use crate::{
    db::Database,
    models::{NewUser, User},
    schema,
    utils::default_head,
};

pub const LOGIN_COOKIE: &str = "AUTHORISED";

#[derive(ThisError, Debug)]
pub enum AuthError {
    #[error("not logged in")]
    NotLoggedIn,
    #[error("invalid cookie state")]
    InvalidCookieIssued,
}

pub struct AuthCookie(pub i32);

impl AuthCookie {
    fn parse(c: Cookie) -> Result<Self, AuthError> {
        let str = c.value();
        match str.parse::<i32>() {
            Ok(t) => Ok(Self(t)),
            Err(_) => Err(AuthError::InvalidCookieIssued),
        }
    }
}

impl FromRequest<'_, '_> for AuthCookie {
    type Error = AuthError;

    fn from_request(
        request: &'_ rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        match request
            .cookies()
            .get_private(LOGIN_COOKIE)
            .map(AuthCookie::parse)
        {
            Some(e) => match e {
                Ok(item) => rocket::request::Outcome::Success(item),
                Err(e) => rocket::request::Outcome::Failure((
                    Status::new(500, "Internal server error."),
                    e,
                )),
            },
            None => rocket::request::Outcome::Failure((
                Status::new(400, "Not logged in."),
                AuthError::NotLoggedIn,
            )),
        }
    }
}

fn login_form() -> malvolio::Form<'static> {
    malvolio::Form::default()
        .attribute("method", "post")
        .child(
            Input::default()
                .attribute("type", "text")
                .attribute("placeholder", "Username")
                .attribute("name", "identifier"),
        )
        .child(Br)
        .child(
            Input::default()
                .attribute("type", "password")
                .attribute("placeholder", "Password")
                .attribute("name", "password"),
        )
        .child(Br)
        .child(
            Input::default()
                .attribute("type", "submit")
                .attribute("value", "Login!"),
        )
}

#[get("/login")]
pub fn login_page() -> Html<'static> {
    Html::default()
        .head(default_head("Login".to_string()))
        .body(Body::default().child(H1::new("Login")).child(login_form()))
}

#[derive(FromForm)]
pub struct LoginData {
    identifier: String,
    password: String,
}

#[post("/login", data = "<data>")]
pub fn login(mut cookies: Cookies, data: Form<LoginData>, conn: Database) -> Html {
    use schema::users::dsl::{email, username, users};
    match users
        .filter(username.eq(&data.identifier))
        .or_filter(email.eq(&data.identifier))
        .first::<User>(&*conn)
    {
        Ok(user) => {
            if verify(&data.password, &user.password)
                .map_err(|e| error!("{:#?}", e))
                .unwrap_or(false)
            {
                cookies.add_private(Cookie::new(LOGIN_COOKIE, user.id.to_string()));
                Html::default()
                    .head(default_head("Logged in".to_string()))
                    .body(
                        Body::default()
                            .child(H1::new("Logged in!"))
                            .child(P::with_text("You are now logged in.")),
                    )
            } else {
                Html::default()
                    .head(default_head("Error".to_string()))
                    .body(
                        Body::default()
                            .child(H1::new("Invalid password"))
                            .child(P::with_text("The password you've supplied isn't correct."))
                            .child(login_form()),
                    )
            }
        }
        Err(error) => match error {
            diesel::result::Error::NotFound => Html::default()
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
            _ => Html::default()
                .head(default_head("Unknown error".to_string()))
                .body(
                    Body::default()
                        .child(H1::new("Database error"))
                        .child(P::with_text(
                            "Something's up on our end. We're working to fix it as fast as
                            we can!",
                        ))
                        .child(login_form()),
                ),
        },
    }
}

fn register_form() -> malvolio::Form<'static> {
    malvolio::Form::default()
        .attribute("method", "post")
        .child(
            Input::default()
                .attribute("type", "text")
                .attribute("placeholder", "Username")
                .attribute("name", "username"),
        )
        .child(Br)
        .child(
            Input::default()
                .attribute("type", "email")
                .attribute("placeholder", "Email")
                .attribute("name", "email"),
        )
        .child(Br)
        .child(
            Input::default()
                .attribute("type", "password")
                .attribute("placeholder", "Password")
                .attribute("name", "password"),
        )
        .child(Br)
        .child(
            Input::default()
                .attribute("type", "password")
                .attribute("placeholder", "Password confirmation")
                .attribute("name", "password_confirmation"),
        )
        .child(Br)
        .child(
            Input::default()
                .attribute("type", "submit")
                .attribute("value", "Login!"),
        )
}

#[get("/register")]
pub fn register_page() -> Html<'static> {
    Html::default()
        .head(default_head("Login".to_string()))
        .body(
            Body::default()
                .child(H1::new("Register"))
                .child(register_form()),
        )
}

#[derive(FromForm)]
pub struct RegisterData {
    username: String,
    email: String,
    password: String,
    password_confirmation: String,
}

lazy_static! {
    static ref EMAIL_RE: Regex =
        Regex::new(r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#).unwrap();
}

#[post("/register", data = "<data>")]
pub fn register(data: Form<RegisterData>, conn: Database, cookies: Cookies) -> Html<'static> {
    use crate::schema::users::dsl::*;
    if cookies.get(LOGIN_COOKIE).is_some() {
        return Html::default()
            .head(default_head("Already logged in".to_string()))
            .body(
                Body::default()
                    .child(H1::new("You are already loggged in."))
                    .child(P::with_text(
                        "It looks like you've just tried to register, but are already logged in.",
                    )),
            );
    };
    if !EMAIL_RE.is_match(&data.email) {
        return Html::default()
            .head(default_head("Invalid email".to_string()))
            .body(
                Body::default()
                    .child(H1::new("Invalid email"))
                    .child(P::with_text("The email provided is not valid."))
                    .child(register_form()),
            );
    }
    if data.password != data.password_confirmation {
        return Html::default()
            .head(default_head("Passwords don't match".to_string()))
            .body(Body::default().child(register_form()));
    }
    let hashed_password = match hash(&data.password, DEFAULT_COST) {
        Ok(string) => string,
        Err(err) => {
            error!("{:#?}", err);
            return Html::default()
                .head(default_head("Encryption error".to_string()))
                .body(
                    Body::default()
                        .child(H1::new("Encryption error"))
                        .child(P::with_text(
                            "We're having problems encrypting your password.",
                        ))
                        .child(register_form()),
                );
        }
    };
    match insert_into(users)
        .values(NewUser::new(
            &data.username,
            &data.email,
            &hashed_password,
            Utc::now().naive_utc(),
        ))
        .get_result::<User>(&*conn)
    {
        Ok(_) => Html::default()
            .head(default_head("You have sucessfully registered!".to_string()))
            .body(
                Body::default()
                    .child(H1::new("Registration successful!"))
                    .child(P::with_text(
                        "We're so happy to have you on board."
                    )),
            ),
        Err(problem) => {
            match problem {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => Html::default()
                    .head(default_head("User already registered".to_string()))
                    .body(
                        Body::default()
                            .child(H1::new("Registration error"))
                            .child(P::with_text(
                                "A user with that username or email already exists."
                            ))
                            .child(register_form()),
                    ),
                _ => {
                    Html::default().head(default_head("Server error".to_string())).body(
                        Body::default()
                            .child(H1::new("Registration error"))
                            .child(P::with_text(
                                "There was an error adding your account to the database. This is not because
                                there is a problem with the data that you have supplied, but because we have
                                made a programming mistake. We're very sorry about this (no really) and are
                                working to resolve it."
                            ))
                            .child(register_form()),
                    )
                }
            }
        }
    }
}

#[get("/logout")]
pub fn logout(mut cookies: Cookies) -> Html {
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
        .body(Body::default().child(H1::new(format!("You are logged out."))))
}

#[get("/reset")]
fn reset() -> Html<'static> {
    todo!()
}

#[post("/reset")]
fn reset_page() -> Html<'static> {
    todo!()
}

#[cfg(test)]
mod test {
    const USERNAME: &str = "user";
    const EMAIL: &str = "user@example.com";
    const PASSWORD: &str = "SecurePasswordWhichM33tsTh3Criteri@";

    use rocket::http::ContentType;

    use super::LOGIN_COOKIE;

    #[test]
    fn test_register_validation() {
        let client = crate::utils::client();
        let mut register_res = client
            .post("/auth/register")
            .header(ContentType::Form)
            .body(format!(
                "username={}&email={}&password={}&password_confirmation={}",
                "something", "an_invalid_email", "validPASSW0RD", "validPASSW0RD"
            ))
            .dispatch();
        let response = register_res.body_string().expect("invalid body response");
        assert!(response.contains("Invalid email"));
    }

    #[test]
    fn test_auth() {
        let client = crate::utils::client();
        // check register page looks right
        let mut register_page = client.get("/auth/register").dispatch();
        let page = register_page.body_string().expect("invalid body response");
        assert!(page.contains("Register"));
        // test can register
        let mut register_res = client
            .post("/auth/register")
            .header(ContentType::Form)
            .body(format!(
                "username={}&email={}&password={}&password_confirmation={}",
                USERNAME, EMAIL, PASSWORD, PASSWORD
            ))
            .dispatch();
        let response = register_res.body_string().expect("invalid body response");
        assert!(response.contains("sucessfully registered"));
        // test login page looks right
        let mut login_page = client.get("/auth/login").dispatch();
        let page = login_page.body_string().expect("invalid body response");
        assert!(page.contains("Login"));
        // test can login
        let mut login_res = client
            .post("/auth/login")
            .header(ContentType::Form)
            .body(format!("identifier={}&password={}", USERNAME, PASSWORD))
            .dispatch();
        // check cookie set
        login_res
            .cookies()
            .into_iter()
            .filter(|c| c.name() == LOGIN_COOKIE)
            .next()
            .unwrap();
        let page = login_res.body_string().expect("invalid body response");
        assert!(page.contains("now logged in"));
    }
}
