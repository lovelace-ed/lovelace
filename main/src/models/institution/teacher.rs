use crate::schema::institution_teacher;
use crate::schema::institution_teacher_invite;

#[derive(Queryable, Identifiable, Debug)]
#[table_name = "institution_teacher"]
pub struct InstitutionTeacher {
    pub id: i32,
    pub user_id: i32,
    pub institution_id: i32,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "institution_teacher"]
pub struct NewInstitutionTeacher {
    pub user_id: i32,
    pub institution_id: i32,
}

#[derive(Queryable, Identifiable, Debug, Clone, Serialize, Deserialize)]
#[table_name = "institution_teacher_invite"]
pub struct InstitutionTeacherInvite {
    pub id: i32,
    pub inviting_user_id: i32,
    pub invited_user_id: i32,
    pub institution_id: i32,
    pub accepted: bool,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "institution_teacher_invite"]
pub struct NewInstitutionTeacherInvite {
    pub inviting_user_id: i32,
    pub invited_user_id: i32,
    pub institution_id: i32,
    pub accepted: bool,
}
