use std::str::FromStr;

use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use diesel::{insert_into, prelude::*};
use malvolio::prelude::*;
use mercutio::Apply;
use portia::form::{FormStyle, FormSubmitInputStyle, FormTextInputStyle};
use regex::Regex;
use rocket::http::CookieJar;
use rocket_contrib::json::Json;
use thiserror::Error as ThisError;

use crate::{
    db::Database,
    email::{EmailBuilder, RecipientBuilder, RecipientsBuilder, SendMail, SendgridMailSender},
    models::{NewUser, User},
    utils::{default_head, json_response::ApiResponse, timezones::timezone_field},
};

use super::{verify::EmailVerificationToken, LOGIN_COOKIE};

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

#[derive(ThisError, Debug)]
pub enum RegisterError {
    #[error("invalid email address supplied")]
    InvalidEmail,
    #[error("passwords do not match")]
    NonMatchingPasswords,
    #[error("already logged in")]
    AlreadyLoggedInError,
    #[error("invalid timezone")]
    InvalidTimezoneError,
    #[error("encrypting password error")]
    EncryptingPasswordError,
    #[error("user already registered")]
    UserAlreadyRegistered,
    #[error("database error")]
    DatabaseError,
}

pub async fn register_base(
    data: &RegisterData,
    conn: Database,
    cookies: &CookieJar<'_>,
) -> Result<User, RegisterError> {
    use crate::schema::users::dsl::*;
    if cookies.get(LOGIN_COOKIE).is_some() {
        return Err(RegisterError::AlreadyLoggedInError);
    };
    let chrono_timezone: chrono_tz::Tz = match FromStr::from_str(data.timezone.trim()) {
        Ok(tz) => tz,
        Err(_) => return Err(RegisterError::InvalidTimezoneError),
    };
    if !EMAIL_RE.is_match(&data.email) {
        return Err(RegisterError::InvalidEmail);
    }
    if data.password != data.password_confirmation {
        return Err(RegisterError::NonMatchingPasswords);
    }
    let hashed_password = match hash(&data.password, DEFAULT_COST) {
        Ok(string) => string,
        Err(err) => {
            error!("{:#?}", err);
            return Err(RegisterError::EncryptingPasswordError);
        }
    };
    let data_clone = data.clone();
    match conn
        .run(move |c| {
            insert_into(users)
                .values(NewUser::new(
                    &data_clone.username,
                    &data_clone.email,
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
                                    .email(user.email.clone())
                                    .name(user.username.clone())
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
            Ok(user)
        }
        Err(problem) => match problem {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => return Err(RegisterError::UserAlreadyRegistered),
            _ => {
                error!("{:#?}", problem);
                return Err(RegisterError::DatabaseError);
            }
        },
    }
}

#[post("/register", data = "<data>")]
pub async fn html_register(
    data: rocket::request::Form<RegisterData>,
    conn: Database,
    cookies: &CookieJar<'_>,
) -> Html {
    match register_base(&data, conn, cookies).await {
        Ok(_) => Html::default()
            .head(default_head("You have sucessfully registered!".to_string()))
            .body(
                Body::default()
                    .child(H1::new("Registration successful!"))
                    .child(P::with_text("We're so happy to have you on board.")),
            ),
        Err(e) => {
            match e {
                RegisterError::InvalidEmail => {
                    Html::default()
                    .head(default_head("Invalid email".to_string()))
                    .body(
                        Body::default()
                            .child(H1::new("Invalid email"))
                            .child(P::with_text("The email provided is not valid."))
                            .child(register_form()),
                    )
                }
                RegisterError::NonMatchingPasswords => {
                    Html::default()
            .head(default_head("Passwords don't match".to_string()))
            .body(Body::default().child(register_form()))
                }
                RegisterError::AlreadyLoggedInError => {
                    Html::default()
                        .head(default_head("Already logged in".to_string()))
                        .body(
                            Body::default()
                                .child(H1::new("You are already loggged in."))
                                .child(P::with_text(
                                    "It looks like you've just tried to register, but are already logged in.",
                                )),
                        )
                }
                RegisterError::InvalidTimezoneError => {
                    Html::default()
                    .head(default_head("Invalid timezone".to_string()))
                    .body(
                        Body::default()
                            .child(H1::new("Invalid timezone"))
                            .child(P::with_text(
                                "Something could be very wrong on our end if this has
                        happened. Please don't hesitate to get in touch if the problem persists.",
                            ))
                            .child(register_form()),
                    )
                }
                RegisterError::EncryptingPasswordError => {
                    Html::default()
                    .head(default_head("Encryption error".to_string()))
                    .body(
                        Body::default()
                            .child(H1::new("Encryption error"))
                            .child(P::with_text(
                                "We're having problems encrypting your password.",
                            ))
                            .child(register_form()),
                    )
                }
                RegisterError::UserAlreadyRegistered => {
                    Html::default()
                .head(default_head("User already registered".to_string()))
                .body(
                    Body::default()
                        .child(H1::new("Registration error"))
                        .child(P::with_text(
                            "A user with that username or email already exists.",
                        ))
                        .child(register_form()),
                )
                }
                RegisterError::DatabaseError => {
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

#[post("/register", data = "<data>")]
pub async fn api_register(
    data: rocket::request::Form<RegisterData>,
    conn: Database,
    cookies: &CookieJar<'_>,
) -> Json<ApiResponse<User>> {
    Json(match register_base(&data, conn, cookies).await {
        Ok(user) => ApiResponse::new_ok(user),
        Err(e) => ApiResponse::new_err(match e {
            RegisterError::InvalidEmail => {
                "The email address provided is not a valid email address."
            }
            RegisterError::NonMatchingPasswords => "The passwords supplied do not match",
            RegisterError::AlreadyLoggedInError => "You are already logged in.",
            RegisterError::InvalidTimezoneError => "The timezone provided is not a valid timezone",
            RegisterError::EncryptingPasswordError => "Could not encrypt the provided password.",
            RegisterError::UserAlreadyRegistered => {
                "A user with that email or username is already registered."
            }
            RegisterError::DatabaseError => {
                "Encountered a database error while undertaking this operation."
            }
        }),
    })
}
