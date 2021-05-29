use chrono::NaiveDateTime;
use diesel::prelude::*;
use malvolio::prelude::*;
use mercutio::Apply;
use portia::{
    form::{FormStyle, FormSubmitInputStyle, FormTextInputStyle},
    render::Render,
};
use rocket::serde::json::Json;

use crate::{
    class::{get_user_role_in_class, tasks::synchronous::AuthCookie, user_is_teacher},
    db::Database,
    models::{ClassSynchronousTask, NewClassSynchronousTask, NewStudentClassSynchronousTask},
    schema::{class_synchronous_task, class_teacher},
    utils::{
        default_head,
        error::{LovelaceError, LovelaceResult},
        error_messages::invalid_date,
        json_response::ApiResponse,
        permission_error::permission_error,
    },
};

/// Create a new form containing the necessary fields to create a new synchronous task.
fn create_new_sync_task_form() -> Form {
    Form::new()
        .apply(FormStyle)
        .child(
            Input::new()
                .attribute(Name::new("title"))
                .attribute(Type::Text),
        )
        .child(
            Input::new()
                .apply(FormTextInputStyle)
                .attribute(Name::new("description"))
                .attribute(Type::Text),
        )
        .child(
            Input::new()
                .apply(FormTextInputStyle)
                .attribute(Name::new("due_date"))
                .attribute(Type::Text),
        )
        .child(
            Input::new()
                .apply(FormSubmitInputStyle)
                .attribute(Type::Submit),
        )
}

#[derive(FromForm, Debug, Clone, Serialize, Deserialize)]
/// The name might give you the impression that this is designed to create a new task to run in a
/// Rust synchronous runtime. It isn't! This is just the form data supplied to the route which is
/// mounted at `class/<class_id>/task/sync/create`
pub struct CreateNewSyncTask {
    title: String,
    description: String,
    start_time: String,
    end_time: String,
}

#[get("/<class_id>/task/sync/create")]
pub async fn get_create_new_sync_task(class_id: i32, auth: AuthCookie, conn: Database) -> Html {
    if conn
        .run(move |c| user_is_teacher(auth.0, class_id, c))
        .await
    {
        Html::new().body(
            Body::new()
                .child(H1::new("Create a new synchronous task."))
                .child(create_new_sync_task_form()),
        )
    } else {
        permission_error()
    }
}

async fn create_new_sync_task(
    conn: Database,
    class_id: i32,
    auth: AuthCookie,
    form: &CreateNewSyncTask,
) -> LovelaceResult<ClassSynchronousTask> {
    match get_user_role_in_class(auth.0, class_id, &conn).await {
        Some(crate::class::ClassMemberRole::Teacher) => {}
        None | Some(crate::class::ClassMemberRole::Student) => {
            return Err(LovelaceError::PermissionError)
        }
    }
    let start_time = match NaiveDateTime::parse_from_str(&form.start_time, "%Y-%m-%dT%H:%M") {
        Ok(date) => date,
        Err(_) => return Err(LovelaceError::ParseDateError),
    };
    let end_time = match NaiveDateTime::parse_from_str(&form.end_time, "%Y-%m-%dT%H:%M") {
        Ok(date) => date,
        Err(_) => return Err(LovelaceError::ParseDateError),
    };
    let title = form.title.clone();
    let description = form.description.clone();
    let task = conn
        .run(move |c| {
            diesel::insert_into(crate::schema::class_synchronous_task::table)
                .values(NewClassSynchronousTask {
                    title: &title,
                    description: &description,
                    created: chrono::Utc::now().naive_utc(),
                    start_time,
                    end_time,
                    class_teacher_id: class_teacher::table
                        .filter(class_teacher::user_id.eq(auth.0))
                        .select(class_teacher::id)
                        .first::<i32>(c)
                        .unwrap(),
                    class_id,
                })
                .returning(class_synchronous_task::all_columns)
                .get_result::<ClassSynchronousTask>(c)
        })
        .await
        .map_err(|e| {
            error!("{:#?}", e);
            LovelaceError::DatabaseError
        })?;
    let student_list = conn
        .run(move |c| {
            crate::schema::class_student::table
                .filter(crate::schema::class_student::class_id.eq(class_id))
                .select(crate::schema::class_student::id)
                .get_results::<i32>(c)
        })
        .await
        .map_err(|e| {
            error!("{:#?}", e);
            LovelaceError::DatabaseError
        })?;
    let class_synchronous_task_id = task.id;
    conn.run(move |c| {
        diesel::insert_into(crate::schema::student_class_synchronous_task::table)
            .values(
                student_list
                    .into_iter()
                    .map(|class_student_id| NewStudentClassSynchronousTask {
                        class_student_id,
                        class_synchronous_task_id,
                    })
                    .collect::<Vec<NewStudentClassSynchronousTask>>(),
            )
            .execute(c)
    })
    .await
    .map_err(|e| {
        error!("{:#?}", e);
        LovelaceError::DatabaseError
    })?;
    Ok(task)
}

#[post("/<class_id>/task/sync/create", data = "<form>")]
pub async fn html_create_new_sync_task(
    conn: Database,
    class_id: i32,
    auth: AuthCookie,
    form: rocket::form::Form<CreateNewSyncTask>,
) -> Html {
    match create_new_sync_task(conn, class_id, auth, &form).await {
        Ok(_) => Html::new()
            .head(default_head("Created that task".to_string()))
            .body(
                Body::new()
                    .child(H1::new("Created that task"))
                    .child(P::with_text("That task has now been sucessfully created.")),
            ),
        Err(e) => {
            if e == LovelaceError::ParseDateError {
                return invalid_date(Some(create_new_sync_task_form()));
            }
            e.render()
        }
    }
}

#[post("/<class_id>/task/sync/create", data = "<form>")]
pub async fn api_create_new_async_task(
    conn: Database,
    class_id: i32,
    auth: AuthCookie,
    form: Json<CreateNewSyncTask>,
) -> Json<ApiResponse<ClassSynchronousTask>> {
    Json(
        match create_new_sync_task(conn, class_id, auth, &form).await {
            Ok(task) => ApiResponse::new_ok(task),
            Err(e) => From::from(e),
        },
    )
}
