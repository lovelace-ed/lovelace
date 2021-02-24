use crate::schema::student_group_student;

#[derive(Queryable, Identifiable, Debug, Clone, Serialize, Deserialize)]
#[table_name = "student_group_student"]
pub struct StudentGroupStudent {
    pub id: i32,
    pub user_id: i32,
    pub student_group_id: i32,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "student_group_student"]
pub struct NewStudentGroupStudent {
    pub user_id: i32,
    pub student_group_id: i32,
}
