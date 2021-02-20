use crate::schema::class_teacher;
use crate::schema::class_teacher_invite;

#[derive(Insertable, Debug, Clone)]
#[table_name = "class_teacher"]
pub struct NewClassTeacher {
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
