/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
use chrono::NaiveDateTime;

use crate::db::DatabaseConnection;
use crate::schema::class_asynchronous_task;
use crate::schema::class_message;
use crate::schema::class_message_reply;
use crate::schema::class_synchronous_task;
use crate::schema::class_teacher;
use crate::schema::class_teacher_invite;
use crate::schema::notifications;
use crate::schema::student_class_asynchronous_task;
use crate::schema::student_class_synchronous_task;
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
    pub fn student_count(id: i32, conn: &DatabaseConnection) -> Result<i64, diesel::result::Error> {
        use crate::schema::class::dsl as class;
        use crate::schema::class_student::dsl as class_student;

        class::class
            .filter(class::id.eq(id))
            .inner_join(class_student::class_student)
            .select(diesel::dsl::count(class_student::id))
            .get_result::<i64>(&*conn)
    }
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "class"]
pub struct NewClass<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub created: NaiveDateTime,
    pub code: &'a str,
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

#[derive(Queryable, Identifiable, Associations, Debug)]
#[belongs_to(Class)]
#[table_name = "class_message"]
pub struct ClassMessage {
    pub id: i32,
    pub title: String,
    pub contents: String,
    pub created_at: NaiveDateTime,
    pub user_id: i32,
    pub class_id: i32,
    pub edited: bool,
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

#[derive(Queryable, Identifiable, Associations, Debug)]
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

#[derive(Queryable, Identifiable, PartialEq, Debug, Clone)]
#[table_name = "class_asynchronous_task"]
pub struct ClassAsynchronousTask {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub created: NaiveDateTime,
    pub due_date: NaiveDateTime,
    pub class_teacher_id: i32,
    pub class_id: i32,
}

impl ClassAsynchronousTask {
    pub fn render(&self) -> malvolio::prelude::Div {
        use malvolio::prelude::*;
        Div::new()
            .child(H3::new(format!("Task: {}", self.title)))
            .child(P::with_text(format!("Description: {}", self.description)))
            .child(P::with_text(format!("Created at: {}", self.created)))
    }
}

#[derive(Insertable)]
#[table_name = "class_asynchronous_task"]
pub struct NewClassAsynchronousTask<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub created: NaiveDateTime,
    pub due_date: NaiveDateTime,
    pub class_teacher_id: i32,
    pub class_id: i32,
}

#[derive(Insertable)]
#[table_name = "student_class_asynchronous_task"]
pub struct NewStudentClassAsynchronousTask {
    pub class_student_id: i32,
    pub class_asynchronous_task_id: i32,
    pub completed: bool,
}

#[derive(Queryable, Identifiable, Associations)]
#[table_name = "student_class_asynchronous_task"]
#[belongs_to(ClassStudent)]
#[belongs_to(ClassAsynchronousTask)]
pub struct StudentClassAsynchronousTask {
    pub id: i32,
    pub class_student_id: i32,
    pub class_asynchronous_task_id: i32,
    pub completed: bool,
}

#[derive(Queryable, Identifiable, PartialEq, Debug, Clone)]
#[table_name = "class_synchronous_task"]
pub struct ClassSynchronousTask {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub created: NaiveDateTime,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub class_teacher_id: i32,
    pub class_id: i32,
}

impl ClassSynchronousTask {
    pub fn render(&self) -> malvolio::prelude::Div {
        use malvolio::prelude::*;
        Div::new()
            .child(H3::new(format!("Task: {}", self.title)))
            .child(P::with_text(format!("Description: {}", self.description)))
            .child(P::with_text(format!("Created at: {}", self.created)))
    }
}

#[derive(Insertable)]
#[table_name = "class_synchronous_task"]
pub struct NewClassSynchronousTask<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub created: NaiveDateTime,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub class_teacher_id: i32,
    pub class_id: i32,
}

#[derive(Insertable)]
#[table_name = "student_class_synchronous_task"]
pub struct NewStudentClassSynchronousTask {
    pub class_student_id: i32,
    pub class_synchronous_task_id: i32,
}

#[derive(Queryable, Identifiable, Associations)]
#[table_name = "student_class_synchronous_task"]
#[belongs_to(ClassStudent)]
#[belongs_to(ClassSynchronousTask)]
pub struct StudentClassSynchronousTask {
    pub id: i32,
    pub class_student_id: i32,
    pub class_synchronous_task_id: i32,
}
