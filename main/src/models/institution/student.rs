use crate::schema::institution_student;
use crate::schema::institution_student_invite;

#[derive(Queryable, Identifiable, Debug)]
#[table_name = "institution_student"]
pub struct InstitutionStudent {
    pub id: i32,
    pub user_id: i32,
    pub institution_id: i32,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "institution_student"]
pub struct NewInstitutionStudent {
    pub user_id: i32,
    pub institution_id: i32,
}

#[derive(Queryable, Identifiable, Debug, Clone, Serialize, Deserialize)]
#[table_name = "institution_student_invite"]
pub struct InstitutionStudentInvite {
    pub id: i32,
    pub inviting_user_id: i32,
    pub invited_user_id: i32,
    pub institution_id: i32,
    pub accepted: bool,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "institution_student_invite"]
pub struct NewInstitutionStudentInvite {
    pub inviting_user_id: i32,
    pub invited_user_id: i32,
    pub institution_id: i32,
    pub accepted: bool,
}
