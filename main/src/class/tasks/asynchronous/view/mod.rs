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

use malvolio::prelude::*;
use rocket::serde::json::Json;
use thiserror::Error as ThisError;

use self::{
    student::{get_student_async_task_summary, render_student_task_summary},
    teacher::{get_teacher_async_task_summary, render_teacher_task_summary},
};

mod student;
mod teacher;

#[derive(ThisError, Debug)]
pub enum ViewAsyncTaskSummaryError {
    #[error("database error")]
    DatabaseError,
}

#[get("/<class_id>/task/async/<task_id>/view")]
/// Retrieve information about a specific asynchronous task.
pub async fn html_view_specific_asynchronous_task(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Html {
    let role = if let Some(role) = get_user_role_in_class(auth.0, class_id, &conn).await {
        role
    } else {
        return permission_error();
    };
    match role {
        crate::class::ClassMemberRole::Teacher => {
            match get_teacher_async_task_summary(task_id, class_id, auth.0, &conn).await {
                Ok((class_task, student_tasks)) => {
                    render_teacher_task_summary(class_task, student_tasks)
                }
                Err(e) => match e {
                    ViewAsyncTaskSummaryError::DatabaseError => database_error(),
                },
            }
        }
        crate::class::ClassMemberRole::Student => {
            match get_student_async_task_summary(task_id, class_id, auth.0, &conn).await {
                Ok((class_task, student_task)) => {
                    render_student_task_summary(class_task, student_task)
                }
                Err(e) => match e {
                    ViewAsyncTaskSummaryError::DatabaseError => database_error(),
                },
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct StudentViewClassRes {
    task: ClassAsynchronousTask,
    student_task: StudentClassAsynchronousTask,
}

#[derive(Serialize, Deserialize)]
pub struct TeacherViewTaskRes {
    task: ClassAsynchronousTask,
    student_tasks: Vec<StudentTask>,
}

#[derive(Serialize, Deserialize)]
pub struct StudentTask {
    student: User,
    task: StudentClassAsynchronousTask,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum ViewSpecificAsynchronousTaskRes {
    Student(StudentViewClassRes),
    Teacher(TeacherViewTaskRes),
}

#[get("/<class_id>/task/async/<task_id>/view")]
/// Retrieve information about a specific asynchronous task.
pub async fn api_view_specific_asynchronous_task(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Json<ApiResponse<ViewSpecificAsynchronousTaskRes>> {
    let role = if let Some(role) = get_user_role_in_class(auth.0, class_id, &conn).await {
        role
    } else {
        return Json(ApiResponse::new_err(
            "You don't have permission to view this task.",
        ));
    };
    Json(match role {
        crate::class::ClassMemberRole::Teacher => {
            match get_teacher_async_task_summary(task_id, class_id, auth.0, &conn).await {
                Ok((class_task, student_tasks)) => ApiResponse::new_ok(
                    ViewSpecificAsynchronousTaskRes::Teacher(TeacherViewTaskRes {
                        task: class_task,
                        student_tasks: student_tasks
                            .into_iter()
                            .map(|(student, task)| StudentTask { student, task })
                            .collect(),
                    }),
                ),
                Err(e) => ApiResponse::new_err(match e {
                    ViewAsyncTaskSummaryError::DatabaseError => {
                        "Encountered an error trying to get this item from the database."
                    }
                }),
            }
        }
        crate::class::ClassMemberRole::Student => {
            match get_student_async_task_summary(task_id, class_id, auth.0, &conn).await {
                Ok((class_task, student_task)) => ApiResponse::new_ok(
                    ViewSpecificAsynchronousTaskRes::Student(StudentViewClassRes {
                        task: class_task,
                        student_task,
                    }),
                ),
                Err(e) => ApiResponse::new_err(match e {
                    ViewAsyncTaskSummaryError::DatabaseError => {
                        "Encountered an error trying to get this item from the database."
                    }
                }),
            }
        }
    })
}
