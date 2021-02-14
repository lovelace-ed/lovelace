/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this exp: (), user_id: () exp: (), user_id: () exp: (), user_id: () exp: (), user_id: () license can be found in the `licenses` directory at the root of this project.
*/
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use diesel::insert_into;
use diesel::prelude::*;
use malvolio::prelude::{
    Body, Br, Href, Html, Input, Method, Name, Placeholder, Type, Value, A, H1, P,
};
use mercutio::Apply;
use portia::form::{FormStyle, FormSubmitInputStyle, FormTextInputStyle};
use regex::Regex;
use rocket::{
    http::{Cookie, CookieJar, RawStr},
    outcome::IntoOutcome,
    request::{Form, FromRequest},
};
use std::str::FromStr;
use thiserror::Error as ThisError;

use crate::{
    db::Database,
    email::{EmailBuilder, RecipientBuilder, RecipientsBuilder, SendMail, SendgridMailSender},
    models::{NewUser, User},
    schema,
    utils::{default_head, error_messages::database_error, timezones::timezone_field},
};

pub const LOGIN_COOKIE: &str = "AUTHORISED";

#[derive(ThisError, Debug)]
pub enum AuthError {}

#[derive(Debug, Copy, Clone)]
pub struct AuthCookie(pub i32);

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for AuthCookie {
    type Error = AuthError;

    async fn from_request(
        request: &'a rocket::Request<'r>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        request
            .cookies()
            .get_private(LOGIN_COOKIE)
            .and_then(|cookie| cookie.value().parse().ok())
            .map(AuthCookie)
            .or_forward(())
    }
}

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

#[derive(FromForm, Debug, Clone)]
pub struct LoginData {
    identifier: String,
    password: String,
}

#[post("/login", data = "<data>")]
pub async fn login(cookies: &CookieJar<'_>, data: Form<LoginData>, conn: Database) -> Html {
    use schema::users::dsl::{email, username, users};
    let closure_data = data.clone();
    match conn
        .run(move |c| {
            users
                .filter(username.eq(&closure_data.identifier))
                .or_filter(email.eq(&closure_data.identifier))
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
                Html::default()
                    .head(default_head("Logged in".to_string()))
                    .body(
                        Body::default()
                            .child(H1::new("Logged in!"))
                            .child(P::with_text("You are now logged in.")),
                    )
            } else {
                Html::default()
                    .status(400)
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
            _ => Html::default()
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

fn register_form() -> malvolio::prelude::Form {
    malvolio::prelude::Form::new()
        .apply(FormStyle)
        .attribute(Method::Post)
        .child(
            Input::default()
                .apply(FormTextInputStyle)
                .attribute(Type::Text)
                .attribute(Placeholder::new("Username"))
                .attribute(Name::new("username")),
        )
        .child(Br)
        .child(
            Input::default()
                .apply(FormTextInputStyle)
                .attribute(Type::Email)
                .attribute(Placeholder::new("Email"))
                .attribute(Name::new("email")),
        )
        .child(Br)
        .child(timezone_field("timezone", None))
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
                .apply(FormTextInputStyle)
                .attribute(Type::Password)
                .attribute(Placeholder::new("Password confirmation"))
                .attribute(Name::new("password_confirmation")),
        )
        .child(Br)
        .child(
            Input::default()
                .apply(FormSubmitInputStyle)
                .attribute(Type::Submit)
                .attribute(Value::new("Login!")),
        )
}

#[get("/register")]
pub fn register_page() -> Html {
    Html::default()
        .head(default_head("Login".to_string()))
        .body(
            Body::default()
                .child(H1::new("Register"))
                .child(register_form()),
        )
}

#[derive(FromForm, Debug, Clone)]
pub struct RegisterData {
    username: String,
    email: String,
    timezone: String,
    password: String,
    password_confirmation: String,
}

lazy_static! {
    static ref EMAIL_RE: Regex =
        Regex::new(r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#).unwrap();
}

#[post("/register", data = "<data>")]
pub async fn register(data: Form<RegisterData>, conn: Database, cookies: &CookieJar<'_>) -> Html {
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
    let chrono_timezone: chrono_tz::Tz = match FromStr::from_str(data.timezone.trim()) {
        Ok(tz) => tz,
        Err(_) => {
            return Html::default()
                .head(default_head("Invalid timezone".to_string()))
                .body(
                    Body::default()
                        .child(H1::new("Invalid timezone"))
                        .child(P::with_text(
                            "Something could be very wrong on our end if this has
                    happened. Please don't hesitate to get in touch if the problem persists.",
                        ))
                        .child(register_form()),
                );
        }
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
    match conn
        .run(move |c| {
            insert_into(users)
                .values(NewUser::new(
                    &data.username,
                    &data.email,
                    &hashed_password,
                    Utc::now().naive_utc(),
                    &chrono_timezone.to_string(),
                ))
                .returning(crate::schema::users::all_columns)
                .get_result::<User>(c)
        })
        .await
    {
        Ok(user) => {
            let email_verification_link = format!(
                "/auth/verify?code={}",
                jwt::encode(
                    &jwt::Header::default(),
                    &EmailVerificationToken {
                        exp: (chrono::Utc::now() + chrono::Duration::days(1)).timestamp() as usize,
                        user_id: user.id
                    },
                    &jwt::EncodingKey::from_base64_secret(
                        &std::env::var("SECRET_KEY").unwrap_or_else(|_| {
                            "NNnXxqFeQ/1Sn8lh9MtlIW2uePR4TL/1O5dB2CPkTmg=".to_string()
                        })
                    )
                    .unwrap(),
                )
                .unwrap()
            );
            let mail_sender = SendgridMailSender::default();
            mail_sender
                .send(
                    &EmailBuilder::default()
                        .subject("Verify your email".to_string())
                        .plaintext(Some(format!(
                            "Copy and paste this link into your browser: {}",
                            email_verification_link
                        )))
                        .html_text(Some(
                            Html::new()
                                .head(default_head("Verify your email".to_string()))
                                .body(
                                    Body::new().child(P::with_text("Verify your email")).child(
                                        A::new().attribute(Href::new(email_verification_link)),
                                    ),
                                )
                                .to_string(),
                        ))
                        .recipients(
                            RecipientsBuilder::default()
                                .recipients(vec![RecipientBuilder::default()
                                    .email(user.email)
                                    .name(user.username)
                                    .build()
                                    .unwrap()])
                                .build()
                                .unwrap(),
                        )
                        .from(("Lovelace".to_string(), "no-reply@lovelace.ga".to_string()))
                        .reply_to(("Lovelace".to_string(), "contact@lovelace.ga".to_string()))
                        .build()
                        .unwrap(),
                )
                .await
                .expect("fatal: failed to send verification email");
            Html::default()
                .head(default_head("You have sucessfully registered!".to_string()))
                .body(
                    Body::default()
                        .child(H1::new("Registration successful!"))
                        .child(P::with_text("We're so happy to have you on board.")),
                )
        }
        Err(problem) => match problem {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => Html::default()
                .head(default_head("User already registered".to_string()))
                .body(
                    Body::default()
                        .child(H1::new("Registration error"))
                        .child(P::with_text(
                            "A user with that username or email already exists.",
                        ))
                        .child(register_form()),
                ),
            _ => {
                error!("{:#?}", problem);
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
        },
    }
}

#[get("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Html {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailVerificationToken {
    pub exp: usize,
    pub user_id: i32,
}

#[get("/verify?<code>")]
pub async fn verify_email(code: &RawStr, conn: Database) -> Html {
    use crate::schema::users::dsl as users;
    match jwt::decode::<EmailVerificationToken>(
        code,
        &jwt::DecodingKey::from_base64_secret(
            &std::env::var("SECRET_KEY")
                .unwrap_or_else(|_| "NNnXxqFeQ/1Sn8lh9MtlIW2uePR4TL/1O5dB2CPkTmg=".to_string()),
        )
        .unwrap(),
        &jwt::Validation::default(),
    ) {
        Ok(code) => {
            match conn
                .run(move |c| {
                    diesel::update(users::users.filter(users::id.eq(code.claims.user_id)))
                        .set(users::email_verified.eq(true))
                        .execute(c)
                })
                .await
            {
                Ok(_) => Html::new()
                    .head(default_head("Email verified".to_string()))
                    .body(Body::new().child(H1::new("Your email has been verified."))),
                Err(_) => database_error(),
            }
        }
        Err(_) => Html::new(),
    }
}

#[get("/reset")]
pub fn reset() -> Html {
    todo!()
}

#[post("/reset")]
pub fn reset_page() -> Html {
    todo!()
}

#[cfg(test)]
mod test {
    const USERNAME: &str = "user";
    const EMAIL: &str = "user@example.com";
    const PASSWORD: &str = "SecurePasswordWhichM33tsTh3Criteri@";
    /// This was chosen for no other reason than it is alphabetically first.
    const TIMEZONE: &str = "Africa/Abidjan";

    use super::EmailVerificationToken;
    use crate::{
        db::Database,
        models::{NewUser, User},
        utils::{client, login_user},
    };
    use diesel::prelude::*;
    use rocket::http::ContentType;
    use wiremock::{
        matchers::{method, path_regex},
        Mock, MockServer, ResponseTemplate,
    };

    #[rocket::async_test]
    async fn test_register_validation() {
        let client = crate::utils::client().await;
        let register_res = client
            .post("/auth/register")
            .header(ContentType::Form)
            .body(format!(
                "username={}&email={}&timezone={timezone}&password={}&password_confirmation={}",
                "something",
                "an_invalid_email",
                "validPASSW0RD",
                "validPASSW0RD",
                timezone = TIMEZONE
            ))
            .dispatch()
            .await;
        let response = register_res
            .into_string()
            .await
            .expect("invalid body response");
        assert!(response.contains("Invalid email"));
    }

    #[rocket::async_test]
    async fn test_auth() {
        let mock_server = MockServer::start().await;
        std::env::set_var("SENDGRID_API_KEY", "SomeRandomAPIKey");
        std::env::set_var("SENDGRID_API_SERVER", mock_server.uri());
        Mock::given(method("post"))
            .and(path_regex("/v3/mail/send"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1..)
            .mount(&mock_server)
            .await;
        let client = rocket::local::asynchronous::Client::tracked(crate::utils::launch())
            .await
            .unwrap();
        // check register page looks right
        let register_page = client.get("/auth/register").dispatch().await;
        let page = register_page
            .into_string()
            .await
            .expect("invalid body response");
        assert!(page.contains("Register"));
        // test can register
        let register_res = client
            .post("/auth/register")
            .header(ContentType::Form)
            .body(format!(
                "username={username}&email={email}&password={password}&timezone={timezone}
                &password_confirmation={password}",
                username = USERNAME,
                email = EMAIL,
                password = PASSWORD,
                timezone = TIMEZONE
            ))
            .dispatch()
            .await;
        let response = register_res
            .into_string()
            .await
            .expect("invalid body response");
        assert!(response.contains("sucessfully registered"));
        // test login page looks right
        let login_page = client.get("/auth/login").dispatch().await;
        let page = login_page
            .into_string()
            .await
            .expect("invalid body response");
        assert!(page.contains("Login"));
        // test can login
        login_user(USERNAME, PASSWORD, &client).await;
    }
    #[rocket::async_test]
    async fn test_email_verification() {
        use crate::schema::users::dsl as users;
        let client = client().await;
        let user_id = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| {
                diesel::insert_into(users::users)
                    .values(NewUser {
                        username: "some-username",
                        email: "email@example.com",
                        password: "123456@#rwefgGFD$TWe",
                        created: chrono::Utc::now().naive_utc(),
                        email_verified: false,
                        timezone: "Africa/Abidjan",
                    })
                    .returning(users::id)
                    .get_result::<i32>(c)
                    .unwrap()
            })
            .await;
        let res = client
            .get(format!(
                "/auth/verify?code={}",
                jwt::encode(
                    &jwt::Header::default(),
                    &EmailVerificationToken {
                        exp: (chrono::Utc::now() + chrono::Duration::days(1)).timestamp() as usize,
                        user_id
                    },
                    &jwt::EncodingKey::from_base64_secret(
                        &std::env::var("SECRET_KEY").unwrap_or_else(|_| {
                            "NNnXxqFeQ/1Sn8lh9MtlIW2uePR4TL/1O5dB2CPkTmg=".to_string()
                        })
                    )
                    .unwrap(),
                )
                .unwrap()
            ))
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains("verified"));
        assert_eq!(
            {
                Database::get_one(&client.rocket())
                    .await
                    .unwrap()
                    .run(move |c| {
                        users::users
                            .filter(users::id.eq(user_id))
                            .first::<User>(c)
                            .unwrap()
                            .email_verified
                    })
                    .await
            },
            true
        )
    }
}
