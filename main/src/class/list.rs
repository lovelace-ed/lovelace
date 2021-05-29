/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

use diesel::prelude::*;
use malvolio::prelude::*;
use mercutio::Apply;
use portia::{levels::Level, render::Render};
use rocket::serde::json::Json;

use crate::utils::error::LovelaceResult;
use crate::utils::{default_head, json_response::ApiResponse};
use crate::{auth::AuthCookie, db::Database};

/// (vec<classes student in>, vec<classes taught>)
async fn view_all_classes(
    auth: AuthCookie,
    conn: Database,
) -> LovelaceResult<(Vec<crate::models::Class>, Vec<crate::models::Class>)> {
    use crate::schema::class::dsl as class;
    use crate::schema::class_student::dsl as class_student;
    use crate::schema::class_teacher::dsl as class_teacher;
    let student_classes = conn
        .run(move |c| {
            class_student::class_student
                .filter(class_student::user_id.eq(auth.0))
                .inner_join(class::class)
                .select(crate::schema::class::all_columns)
                .load::<crate::models::Class>(c)
        })
        .await?;
    let teacher_classes = conn
        .run(move |c| {
            class_teacher::class_teacher
                .filter(class_teacher::user_id.eq(auth.0))
                .inner_join(class::class)
                .select(crate::schema::class::all_columns)
                .load::<crate::models::Class>(c)
        })
        .await?;
    Ok((student_classes, teacher_classes))
}

#[get("/class")]
pub async fn html_view_all_classes(auth_cookie: AuthCookie, conn: Database) -> Html {
    match view_all_classes(auth_cookie, conn).await {
        Ok((student_classes, teacher_classes)) => {
            let student_classes = Level::new()
                .apply(|item| {
                    if !student_classes.is_empty() {
                        item.child(H1::new("Classes I'm a student in".to_string()))
                    } else {
                        item
                    }
                })
                .children(student_classes.iter().map(|class| {
                    Div::new()
                        .child(H3::new(class.name.clone()))
                        .child(P::with_text(class.description.clone()))
                        .child(A::default().attribute(malvolio::prelude::Href::new(format!(
                            "/class/{}",
                            class.id
                        ))))
                }));
            let teacher_classes = Level::new()
                .apply(|item| {
                    if !teacher_classes.is_empty() {
                        item.child(H1::new("Classes I teach"))
                    } else {
                        item
                    }
                })
                .children(teacher_classes.iter().map(|class| {
                    Div::new()
                        .child(H3::new(class.name.clone()))
                        .child(P::with_text(class.description.clone()))
                        .child(
                            A::default()
                                .attribute(Href::new(format!("/class/{}", class.id)))
                                .text("View class"),
                        )
                }));
            Html::default()
                .head(default_head("Classes".to_string()))
                .body(
                    Body::default()
                        .child(teacher_classes)
                        .child(student_classes),
                )
        }
        Err(e) => e.render(),
    }
}

#[derive(Serialize, Deserialize)]
pub struct ViewAllClasses {
    teacher: Vec<crate::models::Class>,
    student: Vec<crate::models::Class>,
}

#[get("/class")]
pub async fn api_view_all_classes(
    conn: Database,
    auth: AuthCookie,
) -> Json<ApiResponse<ViewAllClasses>> {
    Json(match view_all_classes(auth, conn).await {
        Ok((student, teacher)) => ApiResponse::new_ok(ViewAllClasses { teacher, student }),
        Err(e) => From::from(e),
    })
}
