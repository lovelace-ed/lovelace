use chrono::NaiveDateTime;
use malvolio::prelude::*;

use crate::models::ClassStudent;
use crate::schema::class_synchronous_task;
use crate::schema::student_class_synchronous_task;

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
        Div::new()
            .child(H3::new(format!("Task: {}", self.title)))
            .child(P::with_text(format!("Description: {}", self.description)))
            .child(P::with_text(format!("Created at: {}", self.created)))
    }
}

#[derive(Insertable, Clone, Debug)]
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

#[derive(Insertable, Debug, Clone)]
#[table_name = "student_class_synchronous_task"]
pub struct NewStudentClassSynchronousTask {
    pub class_student_id: i32,
    pub class_synchronous_task_id: i32,
}

#[derive(Queryable, Identifiable, Associations, Debug)]
#[table_name = "student_class_synchronous_task"]
#[belongs_to(ClassStudent)]
#[belongs_to(ClassSynchronousTask)]
pub struct StudentClassSynchronousTask {
    pub id: i32,
    pub class_student_id: i32,
    pub class_synchronous_task_id: i32,
}
