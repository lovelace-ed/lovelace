use crate::{
    auth::AuthCookie,
    class::get_user_role_in_class,
    db::Database,
    models::User,
    utils::{default_head, error_message, json_response::ApiResponse},
};

use diesel::prelude::*;
use malvolio::prelude::*;
use portia::levels::Level;
use rocket::serde::json::Json;

#[get("/class/<id>/members")]
pub async fn html_view_class_members_page(
    id: usize,
    conn: Database,
    auth_cookie: AuthCookie,
) -> Html {
    use crate::schema::class::dsl as class;
    use crate::schema::class_student::dsl as class_student;
    use crate::schema::users::dsl as users;
    if get_user_role_in_class(auth_cookie.0 as i32, id as i32, &conn)
        .await
        .is_none()
    {
        return error_message(
            "You don't have permission to view this class.".to_string(),
            "You might need to ask your teacher for a code to join the class.".to_string(),
        );
    };
    let students = conn
        .run(move |c| {
            class::class
                .filter(class::id.eq(id as i32))
                .inner_join(class_student::class_student.inner_join(users::users))
                .select(crate::schema::users::all_columns)
                .load::<User>(c)
        })
        .await
        .map(|users| {
            users
                .into_iter()
                .map(|user| Div::new().child(H3::new(user.username)))
        })
        .unwrap();
    let teachers = conn
        .run(move |c| {
            class::class
                .filter(class::id.eq(id as i32))
                .inner_join(class_student::class_student.inner_join(users::users))
                .select(crate::schema::users::all_columns)
                .load::<User>(c)
        })
        .await
        .map(|users| {
            users
                .into_iter()
                .map(|user| Div::new().child(H3::new(user.username)))
        })
        .unwrap();
    Html::default()
        .head(default_head("Class".to_string()))
        .body(
            Body::default()
                .child(Level::new().child(H3::new("Teachers")).children(teachers))
                .child(Level::new().child(H3::new("Students")).children(students)),
        )
}

#[derive(Serialize, Deserialize)]
pub struct ViewClassMembers {
    teachers: Vec<String>,
    students: Vec<String>,
}

#[get("/class/<id>/members")]
pub async fn api_view_class_members_page(
    id: usize,
    conn: Database,
    auth_cookie: AuthCookie,
) -> Json<ApiResponse<ViewClassMembers>> {
    use crate::schema::class::dsl as class;
    use crate::schema::class_student::dsl as class_student;
    use crate::schema::users::dsl as users;
    if get_user_role_in_class(auth_cookie.0 as i32, id as i32, &conn)
        .await
        .is_none()
    {
        return Json(ApiResponse::new_err(
            "You don't have permission to view this class.
            You might need to ask your teacher for a code to join the class.",
        ));
    };
    let students = conn
        .run(move |c| {
            class::class
                .filter(class::id.eq(id as i32))
                .inner_join(class_student::class_student.inner_join(users::users))
                .select(crate::schema::users::all_columns)
                .load::<User>(c)
        })
        .await
        .map(|users| users.into_iter().map(|user| user.username))
        .unwrap()
        .collect();
    let teachers = conn
        .run(move |c| {
            class::class
                .filter(class::id.eq(id as i32))
                .inner_join(class_student::class_student.inner_join(users::users))
                .select(crate::schema::users::all_columns)
                .load::<User>(c)
        })
        .await
        .map(|users| users.into_iter().map(|user| user.username))
        .unwrap()
        .collect();
    Json(ApiResponse::new_ok(ViewClassMembers { teachers, students }))
}
