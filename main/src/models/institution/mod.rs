use chrono::NaiveDateTime;

use crate::schema::institution;

pub mod administrator;

#[derive(Queryable, Identifiable, Debug, Serialize, Deserialize)]
#[table_name = "institution"]
pub struct Institution {
    pub id: i32,
    pub name: String,
    pub domain: String,
    pub created: NaiveDateTime,
    pub enforce_same_domain: bool,
}

#[derive(Insertable, Debug)]
#[table_name = "institution"]
pub struct NewInstitution<'a> {
    pub name: &'a str,
    pub domain: &'a str,
    pub created: NaiveDateTime,
    pub enforce_same_domain: bool,
}

#[derive(AsChangeset, Debug, Default)]
#[table_name = "institution"]
pub struct UpdateInstitution {
    pub name: Option<String>,
    pub domain: Option<String>,
    pub created: Option<NaiveDateTime>,
    pub enforce_same_domain: Option<bool>,
}
