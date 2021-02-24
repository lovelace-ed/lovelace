use chrono::NaiveDateTime;

use crate::schema::institution;

pub mod administrator;
pub mod student;
pub mod student_group;
pub mod teacher;

#[derive(Queryable, Identifiable, Debug, Serialize, Deserialize)]
#[table_name = "institution"]
pub struct Institution {
    pub id: i32,
    pub name: String,
    pub domain: String,
    pub created: NaiveDateTime,
    pub enforce_same_domain: bool,
    pub let_teachers_create_classes: bool,
    pub let_all_users_create_classes: bool,
    pub let_teachers_add_sync_tasks: bool,
}

#[derive(Insertable, Debug)]
#[table_name = "institution"]
pub struct NewInstitution<'a> {
    pub name: &'a str,
    pub domain: &'a str,
    pub created: NaiveDateTime,
    pub enforce_same_domain: bool,
    pub let_teachers_create_classes: bool,
    pub let_all_users_create_classes: bool,
    pub let_teachers_add_sync_tasks: bool,
}

#[derive(AsChangeset, Debug, Default)]
#[table_name = "institution"]
pub struct UpdateInstitution {
    pub name: Option<String>,
    pub domain: Option<String>,
    pub created: Option<NaiveDateTime>,
    pub enforce_same_domain: Option<bool>,
}
