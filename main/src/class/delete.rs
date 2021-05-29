/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

use diesel::prelude::*;
use malvolio::prelude::*;
use mercutio::Apply;
use portia::form::{FormStyle, FormSubmitInputStyle};
use rocket::serde::json::Json;
use rocket::FromForm;

use crate::{
    auth::AuthCookie,
    db::Database,
    utils::{default_head, error::LovelaceError, error_message, json_response::ApiResponse},
};

fn delete_class_form(id: usize) -> malvolio::prelude::Form {
    malvolio::prelude::Form::new()
        .apply(FormStyle)
        .child(Input::default().attribute(Type::Text))
        .child(
            Input::default()
                .attribute(Type::Hidden)
                .attribute(Name::new("id"))
                .attribute(Value::new(id.to_string())),
        )
        .child(
            Input::default()
                .apply(FormSubmitInputStyle)
                .attribute(Type::Submit)
                .attribute(Value::new(
                    "Delete this class (which I will never be able to get back!)",
                )),
        )
}

#[get("/class/<id>/delete")]
pub fn delete_class_page(id: usize, _auth_cookie: AuthCookie) -> Html {
    Html::default()
        .head(default_head("Delete this class".to_string()))
        .body(
            Body::default()
                .child(H1::new(
                    "Warning – after deleting a class it will be forever gone.",
                ))
                .child(H1::new("This means that you *cannot* get it back."))
                .child(delete_class_form(id)),
        )
}

#[derive(FromForm, Debug, Clone, Serialize, Deserialize)]
pub struct DeleteClassForm {
    id: i32,
    confirm_name: String,
}

#[post("/class/delete", data = "<form>")]
pub async fn html_delete_class(
    form: rocket::form::Form<DeleteClassForm>,
    auth_cookie: AuthCookie,
    conn: Database,
) -> Html {
    use crate::schema::class::dsl as class;
    use crate::schema::class_teacher::dsl as class_teacher;
    let form_id = form.id;
    let user_is_teacher = conn
        .run(move |c| {
            diesel::dsl::select(diesel::dsl::exists(
                class_teacher::class_teacher
                    .filter(class_teacher::user_id.eq(auth_cookie.0 as i32))
                    .filter(class_teacher::class_id.eq(form_id)),
            ))
            .get_result::<bool>(c)
        })
        .await
        .map_err(|e| {
            error!("{:#?}", e);
            e
        });
    if let Ok(is_teacher) = user_is_teacher {
        if !is_teacher {
            return Html::default().head(default_head("Permission denied".to_string())).body(
                Body::default()
                    .child(H1::new("You aren't allowed to do this!"))
                    .child(P::with_text(
                        "You don't have permission to do that because you're not a teacher for this class ."
                    ))
                    .child(delete_class_form(form.id as usize))
            );
        }
    } else {
        return Html::default()
            .head(default_head("Class not found".to_string()))
            .body(
                Body::default()
                    .child(H1::new("We can't find a class with that id"))
                    .child(P::with_text(
                        "Check that the class in question does exist and try again.",
                    ))
                    .child(delete_class_form(form.id as usize)),
            );
    }
    match conn
        .run(move |c| {
            diesel::delete(
                class::class
                    .filter(class::name.eq(&form.confirm_name))
                    .filter(class::id.eq(form.id)),
            )
            .execute(c)
        })
        .await
    {
        Ok(num_deleted) => {
            if num_deleted == 0 {
                return Html::default()
                    .head(default_head("Could not delete this class".to_string()))
                    .body(
                        Body::default()
                            .child(H1::new("Delete this class"))
                            .child(P::with_text(
                                "The name you've typed in doesn't match this class's name.",
                            ))
                            .child(delete_class_form(form_id as usize)),
                    );
            }
            Html::default()
                .head(default_head("Class deleted".to_string()))
                .body(
                    Body::default()
                        .child(H1::new("Class deleted"))
                        .child(P::with_text("That class has been sucessfully deleted.")),
                )
        }
        Err(e) => {
            error!("{:#?}", e);
            error_message(
                "Database error".to_string(),
                "We ran into a database error when trying to delete this task.".to_string(),
            )
        }
    }
}

#[post("/class/delete", data = "<form>")]
pub async fn api_delete_class(
    form: Json<DeleteClassForm>,
    auth_cookie: AuthCookie,
    conn: Database,
) -> Json<ApiResponse<()>> {
    use crate::schema::class::dsl as class;
    use crate::schema::class_teacher::dsl as class_teacher;
    let form_id = form.id;
    let user_is_teacher = conn
        .run(move |c| {
            diesel::dsl::select(diesel::dsl::exists(
                class_teacher::class_teacher
                    .filter(class_teacher::user_id.eq(auth_cookie.0 as i32))
                    .filter(class_teacher::class_id.eq(form_id)),
            ))
            .get_result::<bool>(c)
        })
        .await
        .map_err(|e| {
            error!("{:#?}", e);
            e
        });
    if let Ok(is_teacher) = user_is_teacher {
        if !is_teacher {
            return Json(ApiResponse::new_err(
                "Permission error – you are not a teacher in this class, so you cannot delete the
                class.",
            ));
        }
    } else {
        return Json(ApiResponse::new_err(
            "Could not find a class with that name",
        ));
    }
    Json(
        match conn
            .run(move |c| {
                diesel::delete(
                    class::class
                        .filter(class::name.eq(&form.confirm_name))
                        .filter(class::id.eq(form.id)),
                )
                .execute(c)
            })
            .await
        {
            Ok(num_deleted) => {
                if num_deleted == 0 {
                    ApiResponse::new_err("The name provided does not match that class's name.")
                } else {
                    ApiResponse::new_ok(())
                }
            }
            Err(e) => {
                error!("{:#?}", e);
                LovelaceError::DatabaseError.into()
            }
        },
    )
}
