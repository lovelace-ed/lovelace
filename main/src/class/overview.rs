use malvolio::prelude::*;
use portia::render::Render;
use rocket::serde::json::Json;

use crate::{
    auth::AuthCookie,
    db::Database,
    models::institution::{student_group::StudentGroup, Institution},
    utils::{default_head, error::LovelaceError, json_response::ApiResponse},
};

use super::{get_user_role_in_class, ClassMemberRole};

#[get("/class/<id>")]
pub async fn html_view_class_overview(id: usize, auth_cookie: AuthCookie, conn: Database) -> Html {
    match get_user_role_in_class(auth_cookie.0 as i32, id as i32, &conn).await {
        Some(role) => match role {
            ClassMemberRole::Student => {
                let (class, institution, student_group) =
                    match crate::models::Class::with_institution(id as i32, &conn).await {
                        Ok(t) => t,
                        Err(e) => {
                            error!("{:#?}", e);
                            return LovelaceError::DatabaseError.render();
                        }
                    };
                Html::default()
                    .head(default_head(class.name.to_string()))
                    .body(
                        Body::default()
                            .child(H1::new(format!("Class: {}", class.name)))
                            .map(|body| {
                                if let Some(institution) = institution {
                                    body.child(P::with_text(format!(
                                        "This class is part of {}",
                                        institution.name
                                    )))
                                } else {
                                    body
                                }
                            })
                            .map(|body| {
                                if let Some(student_group) = student_group {
                                    body.child(P::with_text(format!(
                                        "This class is part of student group: {}",
                                        student_group.name
                                    )))
                                } else {
                                    body
                                }
                            })
                            .child(P::with_text(class.description)),
                    )
            }
            ClassMemberRole::Teacher => {
                let (class, institution, student_group) =
                    match crate::models::Class::with_institution(id as i32, &conn).await {
                        Ok(t) => t,
                        Err(e) => {
                            error!("{:#?}", e);
                            return LovelaceError::DatabaseError.render();
                        }
                    };
                Html::default().head(default_head(class.name.clone())).body(
                    Body::default()
                        .child(H1::new(format!("Class: {}", class.name)))
                        .map(|body| {
                            if let Some(institution) = institution {
                                body.child(P::with_text(format!(
                                    "This class is part of {}",
                                    institution.name
                                )))
                            } else {
                                body
                            }
                        })
                        .map(|body| {
                            if let Some(student_group) = student_group {
                                body.child(P::with_text(format!(
                                    "This class is part of student group: {}",
                                    student_group.name
                                )))
                            } else {
                                body
                            }
                        })
                        .child(H3::new(format!(
                            "Invite people to join with the code: {}",
                            class.code
                        )))
                        .child(
                            P::with_text(class.description).child(
                                A::default()
                                    .attribute(Href::new(format!("/class/{}/settings", class.id)))
                                    .text("Settings".to_string()),
                            ),
                        ),
                )
            }
        },
        None => Html::default()
            .head(default_head("Invalid permission".to_string()))
            .body(
                Body::default()
                    .child(H1::new("You don't have permission to view this class."))
                    .child(P::with_text(
                        "You might need to ask your teacher for an invite code.",
                    )),
            ),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudentClass {
    name: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudentOverview {
    class: StudentClass,
    institution: Option<Institution>,
    student_group: Option<StudentGroup>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeacherOverview {
    class: crate::models::Class,
    institution: Option<Institution>,
    student_group: Option<StudentGroup>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClassOverview {
    Teacher(TeacherOverview),
    Student(StudentOverview),
}

#[get("/class/<id>")]
pub async fn api_view_class_overview(
    id: usize,
    auth_cookie: AuthCookie,
    conn: Database,
) -> Json<ApiResponse<ClassOverview>> {
    Json(
        match get_user_role_in_class(auth_cookie.0 as i32, id as i32, &conn).await {
            Some(role) => match role {
                ClassMemberRole::Student => {
                    let (class, institution, student_group) =
                        match crate::models::Class::with_institution(id as i32, &conn).await {
                            Ok(t) => t,
                            Err(e) => {
                                error!("{:#?}", e);
                                return Json(From::from(LovelaceError::DatabaseError));
                            }
                        };
                    ApiResponse::new_ok(ClassOverview::Student(StudentOverview {
                        institution,
                        class: StudentClass {
                            name: class.name,
                            description: class.description,
                        },
                        student_group,
                    }))
                }
                ClassMemberRole::Teacher => {
                    let (class, institution, student_group) =
                        match crate::models::Class::with_institution(id as i32, &conn).await {
                            Ok(t) => t,
                            Err(e) => {
                                error!("{:#?}", e);
                                return Json(From::from(LovelaceError::DatabaseError));
                            }
                        };
                    ApiResponse::new_ok(ClassOverview::Teacher(TeacherOverview {
                        class,
                        institution,
                        student_group,
                    }))
                }
            },
            None => LovelaceError::PermissionError.into(),
        },
    )
}

#[cfg(test)]
pub mod test_class_overview_handling_with_institutions {
    use chrono::Utc;
    use diesel::prelude::*;

    use crate::{
        db::Database,
        models::{
            institution::{
                administrator::NewAdministrator, student_group::NewStudentGroup, NewInstitution,
            },
            NewClass, NewClassStudent, NewClassTeacher, NewUser,
        },
        schema::{
            administrator, class, class_student, class_teacher, institution, student_group, users,
        },
        utils::{client, login_user, logout},
    };

    const TEACHER_USERNAME: &str = "teacher";
    const TEACHER_EMAIL: &str = "teacher@example.com";
    const TEACHER_PASSWORD: &str = "teacherpassw0rD";
    const TIMEZONE: &str = "Africa/Abidjan";

    const STUDENT_USERNAME: &str = "student";
    const STUDENT_EMAIL: &str = "student@example.com";
    const STUDENT_PASSWORD: &str = "studentpassw0rDD";

    const INSTITUTION_NAME: &str = "some institution";
    const INSTITUTION_DOMAIN: &str = "example.com";

    const CLASS_NAME: &str = "class-name";
    const CLASS_DESCRIPTION: &str = "class-description";

    const STUDENT_GROUP_NAME: &str = "student-group";
    const STUDENT_GROUP_DESCRIPTION: &str = "some-description";

    async fn setup_env(conn: Database) -> (i32, i32, i32) {
        conn.run(|c| {
            let teacher_id = diesel::insert_into(users::table)
                .values(NewUser {
                    username: TEACHER_USERNAME,
                    email: TEACHER_EMAIL,
                    password: &bcrypt::hash(TEACHER_PASSWORD, bcrypt::DEFAULT_COST).unwrap(),
                    created: Utc::now().naive_utc(),
                    email_verified: true,
                    timezone: TIMEZONE,
                })
                .returning(users::id)
                .get_result::<i32>(c)
                .unwrap();
            let institution_id = diesel::insert_into(institution::table)
                .values(NewInstitution {
                    name: INSTITUTION_NAME,
                    domain: INSTITUTION_DOMAIN,
                    created: Utc::now().naive_utc(),
                    enforce_same_domain: true,
                    let_teachers_create_classes: true,
                    let_all_users_create_classes: false,
                    let_teachers_add_sync_tasks: true,
                })
                .returning(institution::id)
                .get_result::<i32>(c)
                .unwrap();
            diesel::insert_into(administrator::table)
                .values(NewAdministrator {
                    user_id: teacher_id,
                    institution_id,
                })
                .execute(c)
                .unwrap();
            let student_group_id = diesel::insert_into(student_group::table)
                .values(NewStudentGroup {
                    parent_group: None,
                    institution_id,
                    code: Some(&nanoid!(5)),
                    name: STUDENT_GROUP_NAME.into(),
                    description: STUDENT_GROUP_DESCRIPTION.into(),
                })
                .returning(student_group::id)
                .get_result::<i32>(c)
                .unwrap();
            let class_id = diesel::insert_into(class::table)
                .values(NewClass {
                    name: CLASS_NAME,
                    description: CLASS_DESCRIPTION,
                    created: Utc::now().naive_utc(),
                    code: &nanoid!(5),
                    institution_id: Some(institution_id),
                    student_group_id: Some(student_group_id),
                })
                .returning(class::id)
                .get_result::<i32>(c)
                .unwrap();
            let student_id = diesel::insert_into(users::table)
                .values(NewUser {
                    username: STUDENT_USERNAME,
                    email: STUDENT_EMAIL,
                    password: &bcrypt::hash(STUDENT_PASSWORD, bcrypt::DEFAULT_COST).unwrap(),
                    created: Utc::now().naive_utc(),
                    email_verified: true,
                    timezone: TIMEZONE,
                })
                .returning(users::id)
                .get_result::<i32>(c)
                .unwrap();
            diesel::insert_into(class_student::table)
                .values(NewClassStudent {
                    user_id: student_id,
                    class_id,
                })
                .execute(c)
                .unwrap();
            diesel::insert_into(class_teacher::table)
                .values(NewClassTeacher {
                    user_id: teacher_id,
                    class_id,
                })
                .execute(c)
                .unwrap();

            (institution_id, student_group_id, class_id)
        })
        .await
    }

    #[rocket::async_test]
    async fn test_correct_class_overview() {
        let client = client().await;
        let (_, _, class_id) = setup_env(Database::get_one(client.rocket()).await.unwrap()).await;
        login_user(TEACHER_EMAIL, TEACHER_PASSWORD, &client).await;
        let res = client.get(format!("/class/{}", class_id)).dispatch().await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains(INSTITUTION_NAME));
        assert!(string.contains(STUDENT_GROUP_NAME));
        logout(&client).await;
        login_user(STUDENT_USERNAME, STUDENT_PASSWORD, &client).await;
        login_user(TEACHER_EMAIL, TEACHER_PASSWORD, &client).await;
        let res = client.get(format!("/class/{}", class_id)).dispatch().await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains(INSTITUTION_NAME));
        assert!(string.contains(STUDENT_GROUP_NAME));
    }
}
