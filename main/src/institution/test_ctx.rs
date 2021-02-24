use bcrypt::DEFAULT_COST;
use chrono::Utc;
use diesel::prelude::*;

use crate::{
    db::Database,
    models::{
        institution::{
            administrator::NewAdministrator,
            student::NewInstitutionStudent,
            student_group::{
                student::NewStudentGroupStudent, teacher::NewStudentGroupTeacher, NewStudentGroup,
            },
            teacher::NewInstitutionTeacher,
            NewInstitution,
        },
        NewUser,
    },
    schema::{
        administrator, institution, institution_student, institution_teacher, student_group,
        student_group_student, student_group_teacher, users,
    },
};

pub const ADMIN_USERNAME: &str = "admin";
pub const ADMIN_EMAIL: &str = "admin@example.com";
pub const ADMIN_PASSWORD: &str = "s3cuRE_passw-rd";

pub const TEACHER_USERNAME: &str = "teacher";
pub const TEACHER_EMAIL: &str = "teacher@example.com";

pub const TEACHER_PASSWORD: &str = "teacher-password@12GGWWF";

pub const STUDENT_USERNAME: &str = "student-username";
pub const STUDENT_EMAIL: &str = "student@example.com";
pub const STUDENT_PASSWORD: &str = "student_passw0RD";

pub const TIMEZONE: &str = "Africa/Abidjan";
pub const NAME: &str = "Some educational institution";
pub const WEBSITE: &str = "https://example.com";

pub const STUDENT_GROUP_NAME: &str = "student-group-name";
pub const STUDENT_GROUP_DESCRIPTION: &str = "student-group-description";

/// (user_id, institution_id)
pub async fn setup_env(conn: Database) -> (i32, i32, i32) {
    conn.run(|c| {
        // create users
        let admin_id: i32 = diesel::insert_into(users::table)
            .values(NewUser {
                username: ADMIN_USERNAME,
                email: ADMIN_EMAIL,
                password: &bcrypt::hash(ADMIN_PASSWORD, DEFAULT_COST).unwrap(),
                created: Utc::now().naive_utc(),
                email_verified: true,
                timezone: TIMEZONE,
            })
            .returning(users::id)
            .get_result(c)
            .unwrap();
        let teacher_id = diesel::insert_into(users::table)
            .values(NewUser {
                username: TEACHER_USERNAME,
                email: TEACHER_EMAIL,
                password: &bcrypt::hash(TEACHER_PASSWORD, DEFAULT_COST).unwrap(),
                created: Utc::now().naive_utc(),
                email_verified: true,
                timezone: TIMEZONE,
            })
            .returning(users::id)
            .get_result(c)
            .unwrap();
        let student_id = diesel::insert_into(users::table)
            .values(NewUser {
                username: STUDENT_USERNAME,
                email: STUDENT_EMAIL,
                password: &bcrypt::hash(STUDENT_PASSWORD, DEFAULT_COST).unwrap(),
                created: Utc::now().naive_utc(),
                email_verified: true,
                timezone: TIMEZONE,
            })
            .returning(users::id)
            .get_result(c)
            .unwrap();

        // create institution
        let institution_id: i32 = diesel::insert_into(institution::table)
            .values(NewInstitution {
                name: NAME,
                domain: WEBSITE,
                created: Utc::now().naive_utc(),
                enforce_same_domain: false,
                let_teachers_create_classes: true,
                let_all_users_create_classes: false,
                let_teachers_add_sync_tasks: true,
            })
            .returning(institution::id)
            .get_result(c)
            .unwrap();

        // add relationships
        diesel::insert_into(institution_teacher::table)
            .values(NewInstitutionTeacher {
                user_id: teacher_id,
                institution_id,
            })
            .execute(c)
            .unwrap();
        diesel::insert_into(institution_student::table)
            .values(NewInstitutionStudent {
                user_id: student_id,
                institution_id,
            })
            .execute(c)
            .unwrap();
        diesel::insert_into(administrator::table)
            .values(NewAdministrator {
                user_id: admin_id,
                institution_id,
            })
            .execute(c)
            .unwrap();

        // create student group
        let student_group_id: i32 = diesel::insert_into(student_group::table)
            .values(NewStudentGroup {
                parent_group: None,
                institution_id,
                code: None,
                name: STUDENT_GROUP_NAME,
                description: STUDENT_GROUP_DESCRIPTION,
            })
            .returning(student_group::id)
            .get_result::<i32>(c)
            .unwrap();
        // create student group relationships
        diesel::insert_into(student_group_student::table)
            .values(NewStudentGroupStudent {
                user_id: student_id,
                student_group_id,
            })
            .execute(c)
            .unwrap();
        diesel::insert_into(student_group_teacher::table)
            .values(NewStudentGroupTeacher {
                user_id: teacher_id,
                student_group_id,
            })
            .execute(c)
            .unwrap();
        (admin_id, institution_id, student_group_id)
    })
    .await
}
