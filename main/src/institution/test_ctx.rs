use bcrypt::DEFAULT_COST;
use chrono::Utc;
use diesel::prelude::*;

use crate::{
    db::Database,
    models::{
        institution::{administrator::NewAdministrator, NewInstitution},
        NewUser,
    },
    schema::{administrator, institution, users},
};

pub const USERNAME: &str = "admin";
pub const EMAIL: &str = "admin@example.com";
pub const PASSWORD: &str = "s3cuRE_passw-rd";
pub const TIMEZONE: &str = "Africa/Abidjan";
pub const NAME: &str = "Some educational institution";
pub const WEBSITE: &str = "https://example.com";

/// (user_id, institution_id)
pub async fn setup_env(conn: Database) -> (i32, i32) {
    conn.run(|c| {
        let user_id: i32 = diesel::insert_into(users::table)
            .values(NewUser {
                username: USERNAME,
                email: EMAIL,
                password: &bcrypt::hash(PASSWORD, DEFAULT_COST).unwrap(),
                created: Utc::now().naive_utc(),
                email_verified: true,
                timezone: TIMEZONE,
            })
            .returning(users::id)
            .get_result(c)
            .unwrap();
        let institution_id: i32 = diesel::insert_into(institution::table)
            .values(NewInstitution {
                name: NAME,
                domain: WEBSITE,
                created: Utc::now().naive_utc(),
                enforce_same_domain: false,
            })
            .returning(institution::id)
            .get_result(c)
            .unwrap();
        diesel::insert_into(administrator::table)
            .values(NewAdministrator {
                user_id,
                institution_id,
            })
            .execute(c)
            .unwrap();
        (user_id, institution_id)
    })
    .await
}
