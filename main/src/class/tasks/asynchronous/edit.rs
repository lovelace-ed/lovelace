use crate::{
    catch_database_error,
    class::get_user_role_in_class,
    class::ClassMemberRole,
    models::{ClassAsynchronousTask, UpdateClassAsynchronousTask},
    utils::{
        default_head,
        error_messages::{database_error, invalid_date},
        json_response::ApiResponse,
        permission_error::permission_error,
    },
};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use malvolio::prelude::*;
use mercutio::Apply;
use portia::form::{FormStyle, FormSubmitInputStyle, FormTextInputStyle};
use rocket::serde::json::Json;
use thiserror::Error as ThisError;

use crate::{auth::AuthCookie, db::Database};

fn edit_task_form(
    title: Option<String>,
    description: Option<String>,
    due_date: Option<String>,
) -> Form {
    Form::new()
        .apply(FormStyle)
        .child(
            Input::new()
                .attribute(Type::Text)
                .apply(FormTextInputStyle)
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
                .attribute(Type::Text)
                .apply(FormTextInputStyle)
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
                .apply(FormSubmitInputStyle)
                .map(|item| {
                    if let Some(due_date) = due_date {
                        item.attribute(Value::new(due_date))
                    } else {
                        item
                    }
                })
                .attribute(Name::new("due_date")),
        )
}

#[get("/<class_id>/task/async/<task_id>/edit")]
pub async fn view_edit_task_page(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Html {
    use crate::schema::class_asynchronous_task::dsl as class_asynchronous_task;
    if let Some(role) = get_user_role_in_class(auth.0, class_id, &conn).await {
        if role != ClassMemberRole::Teacher {
            return permission_error();
        }
        let res = catch_database_error!(
            conn.run(move |c| class_asynchronous_task::class_asynchronous_task
                .filter(class_asynchronous_task::id.eq(task_id))
                .first::<ClassAsynchronousTask>(c))
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
                        Some(res.due_date.format("%Y-%m-%dT%H:%M").to_string()),
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
    due_date: String,
}

#[derive(ThisError, Debug)]
pub enum EditTaskError {
    #[error("database error")]
    DatabaseError,
    #[error("permission error")]
    PermissionError,
    #[error("invalid date")]
    InvalidDate,
}

pub async fn apply_edit_task(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
    form: &EditTaskForm,
) -> Result<ClassAsynchronousTask, EditTaskError> {
    use crate::schema::class_asynchronous_task::dsl as class_asynchronous_task;
    let due_date = NaiveDateTime::parse_from_str(&form.due_date, "%Y-%m-%dT%H:%M")
        .map_err(|_| EditTaskError::InvalidDate)?;
    if let Some(role) = get_user_role_in_class(auth.0, class_id, &conn).await {
        if role != ClassMemberRole::Teacher {
            return Err(EditTaskError::DatabaseError);
        }
        let title = Some(form.title.clone());
        let description = Some(form.description.clone());
        conn.run(move |c| {
            diesel::update(
                class_asynchronous_task::class_asynchronous_task
                    .filter(class_asynchronous_task::id.eq(task_id))
                    .filter(class_asynchronous_task::class_id.eq(class_id)),
            )
            .set(UpdateClassAsynchronousTask {
                title,
                description,
                due_date: Some(due_date),
                ..Default::default()
            })
            .returning(crate::schema::class_asynchronous_task::all_columns)
            .get_result(c)
        })
        .await
        .map_err(|e| {
            error!("{:#?}", e);
            EditTaskError::DatabaseError
        })
    } else {
        Err(EditTaskError::PermissionError)
    }
}

#[post("/<class_id>/task/async/<task_id>/edit", data = "<form>")]
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
        Err(e) => match e {
            EditTaskError::DatabaseError => database_error(),
            EditTaskError::PermissionError => permission_error(),
            EditTaskError::InvalidDate => invalid_date(Some(edit_task_form(
                Some(form.title.clone()),
                Some(form.description.clone()),
                Some(form.due_date.clone()),
            ))),
        },
    }
}

#[post("/<class_id>/task/async/<task_id>/edit", data = "<form>")]
pub async fn api_apply_edit_task(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
    form: Json<EditTaskForm>,
) -> Json<ApiResponse<ClassAsynchronousTask>> {
    Json(
        match apply_edit_task(class_id, task_id, auth, conn, &form).await {
            Ok(task) => ApiResponse::new_ok(task),
            Err(e) => ApiResponse::new_err(match e {
                EditTaskError::DatabaseError => "database error",
                EditTaskError::PermissionError => "permission error",
                EditTaskError::InvalidDate => "invalid date",
            }),
        },
    )
}
