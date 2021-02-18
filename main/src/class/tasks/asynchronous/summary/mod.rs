use rocket_contrib::json::Json;
use thiserror::Error as ThisError;

use crate::{
    auth::AuthCookie,
    class::get_user_role_in_class,
    db::Database,
    models::{ClassAsynchronousTask, StudentClassAsynchronousTask, User},
    utils::{
        error_messages::database_error, json_response::ApiResponse,
        permission_error::permission_error,
    },
};
use teacher::{get_teacher_async_tasks_summary, render_teacher_async_tasks};

use malvolio::prelude::*;

use self::student::{get_student_async_tasks_summary, render_student_task_summary};

pub mod student;
pub mod teacher;

#[get("/<class_id>/task/async/all")]
/// Show a list of all the asynchronous tasks have been set in a class, either to a teacher or a
/// student (this is retrieved from the database).
pub async fn html_view_all_async_tasks_in_class(
    class_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Html {
    if let Some(role) = get_user_role_in_class(auth.0, class_id, &conn).await {
        match role {
            crate::class::ClassMemberRole::Teacher => {
                match get_teacher_async_tasks_summary(class_id, auth.0, &conn).await {
                    Ok((tasks, completion_count, student_count)) => {
                        render_teacher_async_tasks(tasks, completion_count, student_count)
                    }
                    Err(e) => match e {
                        ShowAsyncTaskSummaryError::DatabaseError => database_error(),
                    },
                }
            }
            crate::class::ClassMemberRole::Student => {
                let res = match get_student_async_tasks_summary(class_id, auth.0, &conn).await {
                    Ok(res) => res,
                    Err(e) => match e {
                        ShowAsyncTaskSummaryError::DatabaseError => return database_error(),
                    },
                };
                render_student_task_summary(res)
            }
        }
    } else {
        permission_error()
    }
}

#[derive(ThisError, Debug)]
pub enum ShowAsyncTaskSummaryError {
    #[error("database error")]
    DatabaseError,
}

#[derive(Serialize, Deserialize)]
pub struct StudentTasksSummary {
    task: ClassAsynchronousTask,
    student_task: StudentClassAsynchronousTask,
}

#[derive(Serialize, Deserialize)]
pub struct TeacherTasksSummary {
    task: ClassAsynchronousTask,
    set_by: User,
    num_complete: i64,
}

#[derive(Serialize, Deserialize)]
pub struct ClassStudentCount {
    student_count: i64,
    tasks: Vec<TeacherTasksSummary>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum ViewTasksSummary {
    Student(Vec<StudentTasksSummary>),
    Teacher(ClassStudentCount),
}

#[get("/<class_id>/task/async/all")]
/// Show a list of all the asynchronous tasks have been set in a class, either to a teacher or a
/// student (this is retrieved from the database).
pub async fn api_view_all_async_tasks_in_class(
    class_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Json<ApiResponse<ViewTasksSummary>> {
    Json(
        if let Some(role) = get_user_role_in_class(auth.0, class_id, &conn).await {
            match role {
                crate::class::ClassMemberRole::Teacher => {
                    match get_teacher_async_tasks_summary(class_id, auth.0, &conn).await {
                        Ok((tasks, completion_count, student_count)) => {
                            ApiResponse::new_ok(ViewTasksSummary::Teacher(ClassStudentCount {
                                student_count,
                                tasks: tasks
                                    .into_iter()
                                    .zip(completion_count)
                                    .map(|((task, user), completed)| -> TeacherTasksSummary {
                                        TeacherTasksSummary {
                                            task,
                                            set_by: user,
                                            num_complete: completed,
                                        }
                                    })
                                    .collect(),
                            }))
                        }
                        Err(e) => ApiResponse::new_err(match e {
                            ShowAsyncTaskSummaryError::DatabaseError => "database error",
                        }),
                    }
                }
                crate::class::ClassMemberRole::Student => {
                    match get_student_async_tasks_summary(class_id, auth.0, &conn).await {
                        Ok(res) => ApiResponse::new_ok(ViewTasksSummary::Student(
                            res.into_iter()
                                .map(|(student_task, task)| StudentTasksSummary {
                                    task,
                                    student_task,
                                })
                                .collect(),
                        )),
                        Err(e) => ApiResponse::new_err(match e {
                            ShowAsyncTaskSummaryError::DatabaseError => "database error",
                        }),
                    }
                }
            }
        } else {
            ApiResponse::new_err("permission error")
        },
    )
}
