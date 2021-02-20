use crate::schema::{administrator, administrator_invite};

#[derive(Queryable, Identifiable, Serialize, Deserialize, Debug, Clone)]
#[table_name = "administrator"]
pub struct Administrator {
    pub id: i32,
    pub user_id: i32,
    pub institution_id: i32,
}

#[derive(Insertable, Debug, Clone)]
#[table_name = "administrator"]
pub struct NewAdministrator {
    pub user_id: i32,
    pub institution_id: i32,
}

#[derive(Queryable, Identifiable, Debug)]
#[table_name = "administrator_invite"]
pub struct AdministratorInvite {
    pub id: i32,
    pub inviting_user_id: i32,
    pub invited_user_id: i32,
    pub institution_id: i32,
    pub accepted: bool,
}

#[derive(Insertable, Debug)]
#[table_name = "administrator_invite"]
pub struct NewAdministratorInvite {
    pub inviting_user_id: i32,
    pub invited_user_id: i32,
    pub institution_id: i32,
    pub accepted: bool,
}
