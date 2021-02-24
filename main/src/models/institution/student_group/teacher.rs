use crate::schema::student_group_teacher;

#[derive(Queryable, Identifiable, Debug, Clone, Serialize, Deserialize)]
#[table_name = "student_group_teacher"]
pub struct TeacherGroupTeacher {
    pub id: i32,
    pub user_id: i32,
    pub teacher_group_id: i32,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "student_group_teacher"]
pub struct NewStudentGroupTeacher {
    pub user_id: i32,
    pub student_group_id: i32,
}
