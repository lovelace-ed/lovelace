/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this exp: (), user_id: () exp: (), user_id: () exp: (), user_id: () exp: (), user_id: () license can be found in the `licenses` directory at the root of this project.
*/

use rocket::{outcome::IntoOutcome, request::FromRequest};
use thiserror::Error as ThisError;

pub const LOGIN_COOKIE: &str = "AUTHORISED";

mod login;
mod logout;
mod register;
mod reset;
mod verify;

pub use login::{api_login, html_login, login_page};
pub use logout::{api_logout, html_logout_user};
pub use register::{api_register, html_register, register_page};
pub use verify::verify_email;

#[derive(ThisError, Debug)]
pub enum AuthError {}

#[derive(Debug, Copy, Clone)]
pub struct AuthCookie(pub i32);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthCookie {
    type Error = AuthError;

    async fn from_request(
        request: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        request
            .cookies()
            .get_private(LOGIN_COOKIE)
            .and_then(|cookie| cookie.value().parse().ok())
            .map(AuthCookie)
            .or_forward(())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OptionAuthCookie(pub Option<i32>);

impl OptionAuthCookie {
    pub fn none() -> Self {
        Self(None)
    }
}

impl From<AuthCookie> for OptionAuthCookie {
    fn from(cookie: AuthCookie) -> Self {
        OptionAuthCookie(Some(cookie.0))
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for OptionAuthCookie {
    type Error = AuthError;

    async fn from_request(
        request: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        rocket::request::Outcome::Success(OptionAuthCookie(
            request
                .cookies()
                .get_private(LOGIN_COOKIE)
                .and_then(|cookie| cookie.value().parse().ok()),
        ))
    }
}

#[cfg(test)]
mod test_authentication {
    const USERNAME: &str = "user";
    const EMAIL: &str = "user@example.com";
    const PASSWORD: &str = "SecurePasswordWhichM33tsTh3Criteri@";
    /// This was chosen for no other reason than it is alphabetically first.
    const TIMEZONE: &str = "Africa/Abidjan";

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

    use super::verify::EmailVerificationToken;

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
        let user_id = Database::get_one(client.rocket())
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
                Database::get_one(client.rocket())
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
