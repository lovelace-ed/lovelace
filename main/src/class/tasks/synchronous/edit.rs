use chrono::NaiveDateTime;
use diesel::prelude::*;
use malvolio::prelude::*;
use mercutio::Apply;
use portia::form::{FormSubmitInputStyle, FormTextInputStyle};
use portia::{form::FormStyle, render::Render};
use rocket::serde::json::Json;

use crate::{
    catch_database_error,
    class::{get_user_role_in_class, tasks::synchronous::AuthCookie, ClassMemberRole},
    db::Database,
    models::{ClassSynchronousTask, UpdateClassSynchronousTask},
    utils::{
        default_head,
        error::{LovelaceError, LovelaceResult},
        json_response::ApiResponse,
        permission_error::permission_error,
    },
};

fn edit_task_form(
    title: Option<String>,
    description: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Form {
    Form::new()
        .apply(FormStyle)
        .child(
            Input::new()
                .attribute(Type::Text)
                .map(|item| {
                    if let Some(title) = title {
                        item.attribute(Value::new(title))
                    } else {
                        item
                    }
                })
                .attribute(Name::new("title")),
        )
        .child(
            Input::new()
                .apply(FormTextInputStyle)
                .attribute(Type::Text)
                .map(|item| {
                    if let Some(description) = description {
                        item.attribute(Value::new(description))
                    } else {
                        item
                    }
                })
                .attribute(Name::new("description")),
        )
        .child(
            Input::new()
                .attribute(Type::DateTimeLocal)
                .map(|item| {
                    if let Some(start_time) = start_time {
                        item.attribute(Value::new(start_time))
                    } else {
                        item
                    }
                })
                .attribute(Name::new("start_time")),
        )
        .child(
            Input::new()
                .attribute(Type::DateTimeLocal)
                .map(|item| {
                    if let Some(end_time) = end_time {
                        item.attribute(Value::new(end_time))
                    } else {
                        item
                    }
                })
                .attribute(Name::new("end_time")),
        )
        .child(
            Input::new()
                .apply(FormSubmitInputStyle)
                .attribute(Type::Submit),
        )
}

#[get("/<class_id>/task/sync/<task_id>/edit")]
pub async fn view_edit_task_page(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Html {
    use crate::schema::class_synchronous_task::dsl as class_synchronous_task;
    if let Some(role) = get_user_role_in_class(auth.0, class_id, &conn).await {
        if role != ClassMemberRole::Teacher {
            return permission_error();
        }
        let res = catch_database_error!(
            conn.run(move |c| class_synchronous_task::class_synchronous_task
                .filter(class_synchronous_task::id.eq(task_id))
                .first::<ClassSynchronousTask>(c))
                .await
        );
        Html::new()
            .head(default_head("Edit a task".to_string()))
            .body(
                Body::new()
                    .child(H1::new("Edit this task"))
                    .child(edit_task_form(
                        Some(res.title),
                        Some(res.description),
                        Some(res.start_time.format("%Y-%m-%dT%H:%M").to_string()),
                        Some(res.end_time.format("%Y-%m-%dT%H:%M").to_string()),
                    )),
            )
    } else {
        permission_error()
    }
}

#[derive(FromForm, Debug, Clone, Serialize, Deserialize)]
pub struct EditTaskForm {
    title: String,
    description: String,
    start_time: String,
    end_time: String,
}

async fn apply_edit_task(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
    form: &EditTaskForm,
) -> LovelaceResult<ClassSynchronousTask> {
    use crate::schema::class_synchronous_task::dsl as class_synchronous_task;
    if let Some(role) = get_user_role_in_class(auth.0, class_id, &conn).await {
        if role != ClassMemberRole::Teacher {
            return Err(LovelaceError::PermissionError);
        }
        let end_time = match NaiveDateTime::parse_from_str(&form.end_time, "%Y-%m-%dT%H:%M") {
            Ok(date) => date,
            Err(_) => return Err(LovelaceError::ParseDateError),
        };
        let start_time = match NaiveDateTime::parse_from_str(&form.start_time, "%Y-%m-%dT%H:%M") {
            Ok(date) => date,
            Err(_) => return Err(LovelaceError::ParseDateError),
        };
        let title = form.title.clone();
        let description = form.description.clone();
        match conn
            .run(move |c| {
                diesel::update(
                    class_synchronous_task::class_synchronous_task
                        .filter(class_synchronous_task::id.eq(task_id))
                        .filter(class_synchronous_task::class_id.eq(class_id)),
                )
                .set(UpdateClassSynchronousTask {
                    title: Some(&title),
                    description: Some(&description),
                    created: None,
                    start_time: Some(start_time),
                    end_time: Some(end_time),
                    class_teacher_id: None,
                    class_id: None,
                })
                .returning(crate::schema::class_synchronous_task::all_columns)
                .get_result(c)
            })
            .await
        {
            Ok(sync_task) => Ok(sync_task),
            Err(_) => return Err(LovelaceError::DatabaseError),
        }
    } else {
        return Err(LovelaceError::DatabaseError);
    }
}

#[post("/<class_id>/task/sync/<task_id>/edit", data = "<form>")]
pub async fn html_apply_edit_task(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
    form: rocket::form::Form<EditTaskForm>,
) -> Html {
    match apply_edit_task(class_id, task_id, auth, conn, &form).await {
        Ok(_) => Html::new()
            .head(default_head("Successfully updated".to_string()))
            .body(Body::new().child(H1::new("Successfully updated that task."))),
        Err(e) => e.render(),
    }
}

#[post("/<class_id>/task/sync/<task_id>/edit", data = "<form>")]
pub async fn api_apply_edit_task(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
    form: Json<EditTaskForm>,
) -> Json<ApiResponse<ClassSynchronousTask>> {
    Json(
        match apply_edit_task(class_id, task_id, auth, conn, &form).await {
            Ok(task) => ApiResponse::new_ok(task),
            Err(e) => From::from(e),
        },
    )
}
