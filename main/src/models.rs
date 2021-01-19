/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
use chrono::NaiveDateTime;

use crate::schema::class_teacher;
use crate::schema::class_teacher_invite;
use crate::schema::notifications;
use crate::schema::users;
use crate::{db::Database, schema::class};
use crate::{notifications::NotificationPriority, schema::class_student};
use diesel::prelude::*;

#[derive(Queryable, Identifiable, Debug, Clone)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub created: NaiveDateTime,
    pub timezone: String,
    pub email_verified: bool,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "users"]
pub struct NewUser<'a> {
    username: &'a str,
    email: &'a str,
    password: &'a str,
    created: NaiveDateTime,
    email_verified: bool,
    timezone: &'a str,
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

#[derive(Queryable, Identifiable, Debug, Clone)]
#[table_name = "class"]
pub struct Class {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub created: NaiveDateTime,
    pub code: String,
}

impl Class {
    pub fn with_id(id: i32, conn: Database) -> Result<Self, diesel::result::Error> {
        use crate::schema::class::dsl as class;
        class::class.filter(class::id.eq(id)).first::<Self>(&*conn)
    }
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "class"]
pub struct NewClass<'a> {
    name: &'a str,
    description: &'a str,
    created: NaiveDateTime,
    code: &'a str,
}

impl<'a> NewClass<'a> {
    pub fn new(name: &'a str, description: &'a str, created: NaiveDateTime, code: &'a str) -> Self {
        Self {
            name,
            description,
            created,
            code,
        }
    }
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "class_teacher"]
pub struct NewClassTeacher {
    pub user_id: i32,
    pub class_id: i32,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "class_student"]
pub struct NewClassStudent {
    pub user_id: i32,
    pub class_id: i32,
}

#[derive(Queryable, Identifiable, Debug, Clone)]
#[table_name = "class_student"]
pub struct ClassStudent {
    pub id: i32,
    pub user_id: i32,
    pub class_id: i32,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "class_teacher_invite"]
pub struct NewClassTeacherInvite {
    pub inviting_user_id: i32,
    pub invited_user_id: i32,
    pub class_id: i32,
    pub accepted: bool,
}

#[derive(Queryable, Identifiable, Debug, Clone)]
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
