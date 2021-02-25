use chrono::NaiveDateTime;

use crate::models::ClassStudent;
use crate::schema::class_asynchronous_task;
use crate::schema::student_class_asynchronous_task;

#[derive(Queryable, Identifiable, PartialEq, Debug, Clone, Serialize, Deserialize)]
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

#[derive(AsChangeset, Debug, Default)]
#[table_name = "class_asynchronous_task"]
pub struct UpdateClassAsynchronousTask {
    pub title: Option<String>,
    pub description: Option<String>,
    pub created: Option<NaiveDateTime>,
    pub due_date: Option<NaiveDateTime>,
    pub class_teacher_id: Option<i32>,
    pub class_id: Option<i32>,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "class_asynchronous_task"]
pub struct NewClassAsynchronousTask<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub created: NaiveDateTime,
    pub due_date: NaiveDateTime,
    pub class_teacher_id: i32,
    pub class_id: i32,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "student_class_asynchronous_task"]
pub struct NewStudentClassAsynchronousTask {
    pub class_student_id: i32,
    pub class_asynchronous_task_id: i32,
    pub completed: bool,
}

#[derive(
    Queryable,
    Identifiable,
    Associations,
    Debug,
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Clone,
)]
#[table_name = "student_class_asynchronous_task"]
#[belongs_to(ClassStudent)]
#[belongs_to(ClassAsynchronousTask)]
pub struct StudentClassAsynchronousTask {
    pub id: i32,
    pub class_student_id: i32,
    pub class_asynchronous_task_id: i32,
    pub completed: bool,
}
