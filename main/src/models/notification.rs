use chrono::NaiveDateTime;

use crate::{notifications::NotificationPriority, schema::notifications};

#[derive(
    Queryable, Identifiable, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd,
)]
#[table_name = "notifications"]
pub struct Notification {
    pub id: i32,
    pub title: String,
    pub contents: String,
    pub created_at: NaiveDateTime,
    pub priority: i16,
    pub user_id: i32,
    pub read: bool,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "notifications"]
pub struct NewNotification<'a> {
    title: &'a str,
    contents: &'a str,
    created_at: NaiveDateTime,
    priority: i16,
    user_id: i32,
    read: bool,
}

impl<'a> NewNotification<'a> {
    pub fn new(
        title: &'a str,
        contents: &'a str,
        created_at: NaiveDateTime,
        priority: NotificationPriority,
        user_id: i32,
        read: bool,
    ) -> Self {
        Self {
            title,
            contents,
            created_at,
            priority: priority.into(),
            user_id,
            read,
        }
    }
}
