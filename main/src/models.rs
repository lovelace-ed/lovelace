/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
use chrono::NaiveDateTime;

use crate::schema::class_student;
use crate::schema::class_teacher;
use crate::schema::class_teacher_invite;
use crate::schema::users;
use crate::{db::Database, schema::class};
use diesel::prelude::*;

#[derive(Queryable, Identifiable)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub created: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    username: &'a str,
    email: &'a str,
    password: &'a str,
    created: NaiveDateTime,
}

impl<'a> NewUser<'a> {
    pub fn new(
        username: &'a str,
        email: &'a str,
        password: &'a str,
        created: NaiveDateTime,
    ) -> Self {
        NewUser {
            username,
            email,
            password,
            created,
        }
    }
}

#[derive(Queryable, Identifiable)]
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

#[derive(Insertable)]
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

#[derive(Insertable)]
#[table_name = "class_teacher"]
pub struct NewClassTeacher {
    pub user_id: i32,
    pub class_id: i32,
}

#[derive(Insertable)]
#[table_name = "class_student"]
pub struct NewClassStudent {
    pub user_id: i32,
    pub class_id: i32,
}

#[derive(Queryable, Identifiable)]
#[table_name = "class_student"]
pub struct ClassStudent {
    pub id: i32,
    pub user_id: i32,
    pub class_id: i32,
}

#[derive(Insertable)]
#[table_name = "class_teacher_invite"]
pub struct NewClassTeacherInvite {
    pub inviting_user_id: i32,
    pub invited_user_id: i32,
    pub class_id: i32,
    pub accepted: bool,
}
