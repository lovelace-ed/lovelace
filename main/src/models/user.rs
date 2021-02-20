use chrono::NaiveDateTime;

use crate::schema::users;

#[derive(Queryable, Identifiable, Debug, Clone, Serialize, Deserialize)]
#[table_name = "users"]
/// Note: this struct cannot be deserialized as-is (because we never serialize confidential data –
/// i.e. password hashes, emails and whether or not the user has verified their email).
pub struct User {
    pub id: i32,
    pub username: String,
    #[serde(skip_serializing)]
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub created: NaiveDateTime,
    pub timezone: String,
    #[serde(skip_serializing)]
    pub email_verified: bool,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub password: &'a str,
    pub created: NaiveDateTime,
    pub email_verified: bool,
    pub timezone: &'a str,
}

impl<'a> NewUser<'a> {
    pub fn new(
        username: &'a str,
        email: &'a str,
        password: &'a str,
        created: NaiveDateTime,
        timezone: &'a str,
    ) -> Self {
        NewUser {
            username,
            email,
            password,
            created,
            timezone,
            email_verified: false,
        }
    }
}
