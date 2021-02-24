use diesel::prelude::*;
use malvolio::prelude::*;
use portia::render::Render;
use rocket_contrib::json::Json;

use crate::{
    auth::AuthCookie,
    class::get_user_role_in_class,
    css_names::{LIST, LIST_ITEM},
    db::Database,
    models::{ClassSynchronousTask, StudentClassSynchronousTask, User},
    utils::{
        default_head,
        error::{LovelaceError, LovelaceResult},
        json_response::ApiResponse,
        permission_error::permission_error,
    },
};

/// Show a list of all the tasks in a class that a student has been assigned.
async fn show_student_sync_tasks_summary(
    class_id: i32,
    user_id: i32,
    conn: &Database,
) -> LovelaceResult<Vec<(StudentClassSynchronousTask, ClassSynchronousTask)>> {
    use crate::schema::class_student::dsl as class_student;
    use crate::schema::class_synchronous_task::dsl as class_synchronous_task;
    use crate::schema::student_class_synchronous_task::dsl as student_class_synchronous_task;

    match conn
        .run(move |c| {
            student_class_synchronous_task::student_class_synchronous_task
                .inner_join(class_student::class_student)
                .filter(class_student::user_id.eq(user_id))
                .inner_join(class_synchronous_task::class_synchronous_task)
                .filter(class_synchronous_task::class_id.eq(class_id))
                .select((
                    crate::schema::student_class_synchronous_task::all_columns,
                    crate::schema::class_synchronous_task::all_columns,
                ))
                .load::<(StudentClassSynchronousTask, ClassSynchronousTask)>(c)
        })
        .await
    {
        Ok(tasks) => Ok(tasks),
        Err(e) => {
            error!("{:#?}", e);
            Err(LovelaceError::DatabaseError)
        }
    }
}

struct RenderClassTaskList(pub Vec<(StudentClassSynchronousTask, ClassSynchronousTask)>);

impl Render<Html> for RenderClassTaskList {
    fn render(self) -> Html {
        if self.0.is_empty() {
            Html::new()
                .head(default_head("No tasks found.".to_string()))
                .body(Body::new().child(H1::new("No tasks have been set for this class yet.")))
        } else {
            Html::new()
                .head(default_head("Tasks for this class".to_string()))
                .body(
                    Body::new().child(H1::new("Tasks for this class")).child(
                        Div::new()
                            .attribute(Class::from(LIST))
                            .children(self.0.into_iter().map(|(_, class_task_instance)| {
                                Div::new()
                                    .child(H3::new(format!("Task: {}", class_task_instance.title)))
                                    .child(P::with_text(format!(
                                        "Description: {}",
                                        class_task_instance.description
                                    )))
                            })),
                    ),
                )
        }
    }
}

/// Show the list of tasks that have been set in a class. At some point we'll want to add pagination
/// support for this.
///
/// MAKE SURE YOU HAVE CHECKED THAT THE USER IS A TEACHER IN THE CLASS BEFORE YOU CALL THIS
/// FUNCTION. (sorry for the all caps, I (@teymour-aldridge) kept forgetting to do so :-)
async fn show_teacher_sync_tasks_summary(
    class_id: i32,
    conn: &Database,
) -> LovelaceResult<Vec<(ClassSynchronousTask, User)>> {
    use crate::schema::class_synchronous_task::dsl as class_synchronous_task;
    use crate::schema::class_teacher::dsl as class_teacher;
    use crate::schema::student_class_synchronous_task::dsl as student_class_synchronous_task;
    let query = class_synchronous_task::class_synchronous_task
        .filter(class_synchronous_task::class_id.eq(class_id))
        .order_by(class_synchronous_task::start_time.desc())
        .inner_join(student_class_synchronous_task::student_class_synchronous_task);
    conn.run(move |c| {
        query
            .inner_join(class_teacher::class_teacher.inner_join(crate::schema::users::dsl::users))
            .select((
                crate::schema::class_synchronous_task::all_columns,
                crate::schema::users::all_columns,
            ))
            .load::<(ClassSynchronousTask, User)>(c)
    })
    .await
    .map_err(|e| {
        error!("{:#?}", e);
        LovelaceError::DatabaseError
    })
}

struct RenderTeacherTaskList(pub Vec<(ClassSynchronousTask, User)>);

impl Render<Html> for RenderTeacherTaskList {
    fn render(self) -> Html {
        Html::new().head(default_head("Tasks".to_string())).body(
            Body::new().child(Div::new().attribute(Class::from(LIST)).children(
                self.0.into_iter().map(|(task, set_by)| {
                    Div::new()
                        .attribute(Class::from(LIST_ITEM))
                        .child(task.render())
                        .child(P::with_text(format!("Set by: {}", set_by.username)))
                }),
            )),
        )
    }
}

#[get("/<class_id>/task/sync/all")]
/// Show a list of all the synchronous tasks have been set in a class, either to a teacher or a
/// student (this is retrieved from the database).
pub async fn html_view_all_sync_tasks_in_class(
    class_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Html {
    if let Some(role) = get_user_role_in_class(auth.0, class_id, &conn).await {
        match role {
            crate::class::ClassMemberRole::Teacher => {
                let tasks = match show_teacher_sync_tasks_summary(class_id, &conn).await {
                    Ok(t) => t,
                    Err(e) => return e.render(),
                };
                RenderTeacherTaskList(tasks).render()
            }
            crate::class::ClassMemberRole::Student => {
                let tasks = match show_student_sync_tasks_summary(class_id, auth.0, &conn).await {
                    Ok(t) => t,
                    Err(e) => return e.render(),
                };
                RenderClassTaskList(tasks).render()
            }
        }
    } else {
        permission_error()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeacherTask {
    task: ClassSynchronousTask,
    user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudentTask {
    task: ClassSynchronousTask,
    student_task: StudentClassSynchronousTask,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ViewAllSyncTasks {
    Teacher(Vec<TeacherTask>),
    Student(Vec<StudentTask>),
}

#[get("/<class_id>/task/sync/all")]
/// Show a list of all the synchronous tasks have been set in a class, either to a teacher or a
/// student (this is retrieved from the database).
pub async fn api_view_all_sync_tasks_in_class(
    class_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Json<ApiResponse<ViewAllSyncTasks>> {
    Json(
        if let Some(role) = get_user_role_in_class(auth.0, class_id, &conn).await {
            match role {
                crate::class::ClassMemberRole::Teacher => {
                    let tasks = match show_teacher_sync_tasks_summary(class_id, &conn).await {
                        Ok(t) => t,
                        Err(e) => return Json(e.into()),
                    };
                    ApiResponse::new_ok(ViewAllSyncTasks::Teacher(
                        tasks
                            .into_iter()
                            .map(|(task, user)| TeacherTask { task, user })
                            .collect(),
                    ))
                }
                crate::class::ClassMemberRole::Student => {
                    let tasks = match show_student_sync_tasks_summary(class_id, auth.0, &conn).await
                    {
                        Ok(t) => t,
                        Err(e) => return Json(e.into()),
                    };
                    ApiResponse::new_ok(ViewAllSyncTasks::Student(
                        tasks
                            .into_iter()
                            .map(|(student_task, task)| StudentTask { task, student_task })
                            .collect(),
                    ))
                }
            }
        } else {
            return Json(From::from(LovelaceError::PermissionError));
        },
    )
}
