pub mod student;
pub mod teacher;

use crate::schema::student_group;

#[derive(Queryable, Identifiable, Debug, Clone, Serialize, Deserialize)]
#[table_name = "student_group"]
pub struct StudentGroup {
    pub id: i32,
    pub parent_group: Option<i32>,
    pub institution_id: i32,
    pub code: Option<String>,
    pub name: String,
    pub description: String,
}

#[derive(Insertable, Debug)]
#[table_name = "student_group"]
pub struct NewStudentGroup<'a> {
    pub parent_group: Option<i32>,
    pub institution_id: i32,
    pub code: Option<&'a str>,
    pub name: &'a str,
    pub description: &'a str,
}

#[derive(AsChangeset, Debug)]
#[table_name = "student_group"]
pub struct UpdateStudentGroup {
    pub parent_group: Option<Option<i32>>,
    pub institution_id: i32,
    pub code: Option<Option<String>>,
    pub name: Option<String>,
    pub description: Option<String>,
}
