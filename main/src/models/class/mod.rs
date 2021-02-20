use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::db::Database;
use crate::schema::class;

pub mod async_task;
pub mod message;
pub mod student;
pub mod sync_task;
pub mod teacher;

pub use async_task::*;
pub use message::*;
pub use student::*;
pub use sync_task::*;
pub use teacher::*;

#[derive(Queryable, Identifiable, Debug, Clone, Serialize, Deserialize)]
#[table_name = "class"]
pub struct Class {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub created: NaiveDateTime,
    #[serde(skip_serializing)]
    pub code: String,
    pub institution_id: Option<i32>,
    pub student_group_id: Option<i32>,
}

impl Class {
    /// Returns an instance of the class with the given id.
    pub async fn with_id(id: i32, conn: &Database) -> Result<Self, diesel::result::Error> {
        use crate::schema::class::dsl as class;
        conn.run(move |c| class::class.filter(class::id.eq(id)).first::<Self>(c))
            .await
    }
    /// Returns the number of students in the class in question.
    pub async fn student_count(id: i32, conn: &Database) -> Result<i64, diesel::result::Error> {
        use crate::schema::class::dsl as class;
        use crate::schema::class_student::dsl as class_student;

        conn.run(move |c| {
            class::class
                .filter(class::id.eq(id))
                .inner_join(class_student::class_student)
                .select(diesel::dsl::count(class_student::id))
                .get_result::<i64>(c)
        })
        .await
    }
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "class"]
pub struct NewClass<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub created: NaiveDateTime,
    pub code: &'a str,
    pub institution_id: Option<i32>,
    pub student_group_id: Option<i32>,
}

impl<'a> NewClass<'a> {
    pub fn new(
        name: &'a str,
        description: &'a str,
        created: NaiveDateTime,
        code: &'a str,
        institution_id: Option<i32>,
        student_group_id: Option<i32>,
    ) -> Self {
        Self {
            name,
            description,
            created,
            code,
            institution_id,
            student_group_id,
        }
    }
}
