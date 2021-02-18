use crate::calendar::scheduler::schedule_class;
use crate::class::user_is_teacher;
use crate::models::ClassAsynchronousTask;
use crate::models::NewClassAsynchronousTask;
use crate::models::NewStudentClassAsynchronousTask;
use crate::utils::default_head;
use crate::utils::error_messages::database_error;
use crate::utils::error_messages::invalid_date;
use crate::utils::permission_error::permission_error;
use crate::{auth::AuthCookie, db::Database};
use crate::{class::get_user_role_in_class, utils::json_response::ApiResponse};

use chrono::Duration;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use malvolio::prelude::*;
use mercutio::Apply;
use portia::form::FormStyle;
use rocket_contrib::json::Json;
use thiserror::Error as ThisError;

/// Create a new form containing the necessary fields to create a new asynchronous task.
fn create_new_async_task_form() -> Form {
    Form::new()
        .apply(FormStyle)
        .child(
            Input::new()
                .attribute(Name::new("title"))
                .attribute(Type::Text),
        )
        .child(
            Input::new()
                .attribute(Name::new("description"))
                .attribute(Type::Text),
        )
        .child(
            Input::new()
                .attribute(Name::new("due_date"))
                .attribute(Type::Text),
        )
        .child(Input::new().attribute(Type::Submit))
}

#[derive(FromForm, Debug, Clone, Serialize, Deserialize)]
/// The name might give you the impression that this is designed to create a new task to run in a
/// Rust asynchronous runtime. It isn't! This is just the form data supplied to the route which is
/// mounted at `class/<class_id>/task/async/create`
pub struct CreateNewAsyncTask {
    title: String,
    description: String,
    due_date: String,
}

#[get("/<class_id>/task/async/create")]
pub async fn get_create_new_async_task(class_id: i32, auth: AuthCookie, conn: Database) -> Html {
    if conn
        .run(move |c| user_is_teacher(auth.0, class_id, c))
        .await
    {
        Html::new().body(
            Body::new()
                .child(H1::new("Create a new asynchronous task."))
                .child(create_new_async_task_form()),
        )
    } else {
        permission_error()
    }
}

#[derive(ThisError, Debug)]
pub enum CreateAsyncTaskError {
    #[error("database error")]
    DatabaseError,
    #[error("permission error")]
    PermissionError,
    #[error("invalid date")]
    InvalidDate,
}

async fn new_async_task(
    conn: Database,
    class_id: i32,
    auth: AuthCookie,
    form: &CreateNewAsyncTask,
) -> Result<ClassAsynchronousTask, CreateAsyncTaskError> {
    use crate::schema::class_teacher::dsl as class_teacher;
    match get_user_role_in_class(auth.0, class_id, &conn).await {
        Some(crate::class::ClassMemberRole::Teacher) => {}
        None | Some(crate::class::ClassMemberRole::Student) => {
            return Err(CreateAsyncTaskError::PermissionError)
        }
    };
    let due_date = match NaiveDateTime::parse_from_str(&form.due_date, "%Y-%m-%dT%H:%M") {
        Ok(date) => date,
        Err(_) => return Err(CreateAsyncTaskError::InvalidDate),
    };
    let title = form.title.clone();
    let description = form.description.clone();
    match conn
        .run(move |c| {
            diesel::insert_into(crate::schema::class_asynchronous_task::table)
                .values(NewClassAsynchronousTask {
                    title: &title,
                    description: &description,
                    created: chrono::Utc::now().naive_utc(),
                    due_date,
                    class_teacher_id: class_teacher::class_teacher
                        .filter(class_teacher::user_id.eq(auth.0))
                        .select(class_teacher::id)
                        .first::<i32>(c)
                        .unwrap(),
                    class_id,
                })
                .returning(crate::schema::class_asynchronous_task::all_columns)
                .get_result::<ClassAsynchronousTask>(c)
        })
        .await
    {
        Ok(async_task) => {
            let async_task_id = async_task.id;
            let student_list = match conn
                .run(move |c| {
                    crate::schema::class_student::table
                        .filter(crate::schema::class_student::class_id.eq(class_id))
                        .select(crate::schema::class_student::id)
                        .get_results::<i32>(c)
                })
                .await
            {
                Ok(t) => t,
                Err(_) => return Err(CreateAsyncTaskError::DatabaseError),
            };
            match conn
                .run(move |c| {
                    diesel::insert_into(crate::schema::student_class_asynchronous_task::table)
                        .values(
                            student_list
                                .into_iter()
                                .map(|class_student_id| NewStudentClassAsynchronousTask {
                                    class_student_id,
                                    class_asynchronous_task_id: async_task_id,
                                    completed: false,
                                })
                                .collect::<Vec<NewStudentClassAsynchronousTask>>(),
                        )
                        .execute(c)
                })
                .await
            {
                Ok(_) => {
                    if due_date < Utc::now().naive_utc() + Duration::days(14) {
                        rocket::tokio::spawn(async move {
                            let _ = schedule_class(class_id, &conn).await;
                        });
                    }
                    Ok(async_task)
                }
                Err(e) => {
                    error!("{:#?}", e);
                    return Err(CreateAsyncTaskError::DatabaseError);
                }
            }
        }
        Err(e) => {
            error!("{:#?}", e);
            return Err(CreateAsyncTaskError::DatabaseError);
        }
    }
}

#[post("/<class_id>/task/async/create", data = "<form>")]
pub async fn html_create_new_async_task(
    conn: Database,
    class_id: i32,
    auth: AuthCookie,
    form: rocket::request::Form<CreateNewAsyncTask>,
) -> Html {
    match new_async_task(conn, class_id, auth, &form).await {
        Ok(_) => Html::new()
            .head(default_head("Created that task".to_string()))
            .body(
                Body::new()
                    .child(H1::new("Created that task"))
                    .child(P::with_text("That task has now been sucessfully created.")),
            ),
        Err(e) => match e {
            CreateAsyncTaskError::DatabaseError => database_error(),
            CreateAsyncTaskError::PermissionError => permission_error(),
            CreateAsyncTaskError::InvalidDate => {
                return invalid_date(Some(create_new_async_task_form()))
            }
        },
    }
}

#[post("/<class_id>/task/async/create", data = "<form>")]
pub async fn api_create_new_async_task(
    conn: Database,
    class_id: i32,
    auth: AuthCookie,
    form: Json<CreateNewAsyncTask>,
) -> Json<ApiResponse<ClassAsynchronousTask>> {
    Json(match new_async_task(conn, class_id, auth, &form).await {
        Ok(task) => ApiResponse::new_ok(task),
        Err(e) => ApiResponse::new_err(match e {
            CreateAsyncTaskError::DatabaseError => {
                "Encountered a database error when trying to fulfill this operation."
            }
            CreateAsyncTaskError::PermissionError => {
                "You don't have permissions to create tasks in this class."
            }
            CreateAsyncTaskError::InvalidDate => "The date you provided is not in a valid format.",
        }),
    })
}
