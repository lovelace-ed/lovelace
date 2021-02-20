use crate::models::Class;
use crate::models::User;
use chrono::NaiveDateTime;

use crate::schema::class_message;
use crate::schema::class_message_reply;

#[derive(
    Queryable,
    Identifiable,
    Associations,
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
)]
#[belongs_to(Class)]
#[table_name = "class_message"]
pub struct ClassMessage {
    pub id: i32,
    pub title: String,
    pub contents: String,
    pub created_at: NaiveDateTime,
    #[serde(skip_serializing)]
    pub user_id: i32,
    #[serde(skip_serializing)]
    pub class_id: i32,
    pub edited: bool,
}

#[derive(AsChangeset, Default, Debug)]
#[table_name = "class_message"]
pub struct UpdateClassMessage {
    pub title: Option<String>,
    pub contents: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub user_id: Option<i32>,
    pub class_id: Option<i32>,
    pub edited: Option<bool>,
}

#[derive(Insertable, Debug)]
#[table_name = "class_message"]
pub struct NewClassMessage<'a> {
    pub title: &'a str,
    pub contents: &'a str,
    pub created_at: NaiveDateTime,
    pub user_id: i32,
    pub class_id: i32,
    pub edited: bool,
}

#[derive(Queryable, Identifiable, Associations, Debug, Serialize, Deserialize)]
#[table_name = "class_message_reply"]
#[belongs_to(User)]
#[belongs_to(ClassMessage)]
pub struct ClassMessageReply {
    pub id: i32,
    pub contents: String,
    pub created_at: NaiveDateTime,
    pub edited: bool,
    pub user_id: i32,
    pub class_id: i32,
    pub class_message_id: i32,
}

#[derive(AsChangeset, Debug, Default)]
#[table_name = "class_message_reply"]
pub struct UpdateClassMessageReply {
    pub contents: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub edited: Option<bool>,
    pub user_id: Option<i32>,
    pub class_message_id: Option<i32>,
}

#[derive(Insertable, Debug)]
#[table_name = "class_message_reply"]
pub struct NewClassMessageReply<'a> {
    pub contents: &'a str,
    pub created_at: NaiveDateTime,
    pub edited: bool,
    pub user_id: i32,
    pub class_id: i32,
    pub class_message_id: i32,
}
