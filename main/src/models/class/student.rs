use crate::schema::class_student;

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
