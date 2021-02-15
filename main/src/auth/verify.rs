use diesel::prelude::*;
use malvolio::prelude::*;
use rocket::http::RawStr;

use crate::{
    db::Database,
    utils::{default_head, error_messages::database_error},
};

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
