use diesel::prelude::*;
use malvolio::prelude::*;
use portia::{levels::Level, render::Render};
use rocket_contrib::json::Json;

use crate::{
    auth::AuthCookie,
    class::get_user_role_in_class,
    db::Database,
    models::{sync_task, user, ClassSynchronousTask, StudentClassSynchronousTask, User},
    utils::{
        default_head,
        error::{LovelaceError, LovelaceResult},
        json_response::ApiResponse,
        permission_error::permission_error,
    },
};

async fn get_student_sync_task_summary(
    task_id: i32,
    class_id: i32,
    user_id: i32,
    conn: &Database,
) -> LovelaceResult<(ClassSynchronousTask, StudentClassSynchronousTask)> {
    use crate::schema::class_student::dsl as class_student;
    use crate::schema::class_synchronous_task::dsl as class_synchronous_task;
    conn.run(move |c| {
        crate::schema::student_class_synchronous_task::table
            .inner_join(class_synchronous_task::class_synchronous_task)
            .filter(class_synchronous_task::id.eq(task_id))
            .filter(class_synchronous_task::class_id.eq(class_id))
            .inner_join(class_student::class_student)
            .filter(class_student::user_id.eq(user_id))
            .filter(class_student::class_id.eq(class_id))
            .select((
                crate::schema::class_synchronous_task::all_columns,
                crate::schema::student_class_synchronous_task::all_columns,
            ))
            .first::<(ClassSynchronousTask, StudentClassSynchronousTask)>(c)
    })
    .await
    .map_err(|e| {
        error!("{:#?}", e);
        LovelaceError::DatabaseError
    })
}

pub struct RenderStudentSummary(pub (ClassSynchronousTask, StudentClassSynchronousTask));

impl Render<Html> for RenderStudentSummary {
    fn render(self) -> Html {
        let (class_task, _) = self.0;
        Html::new().head(default_head("Task".to_string())).body(
            Body::new()
                .child(H1::new(format!("Task {}", class_task.title)))
                .child(P::with_text(format!(
                    "Description {}",
                    class_task.description
                ))),
        )
    }
}

async fn get_teacher_sync_task_summary(
    task_id: i32,
    class_id: i32,
    user_id: i32,
    conn: &Database,
) -> LovelaceResult<(
    ClassSynchronousTask,
    Vec<(user::User, sync_task::StudentClassSynchronousTask)>,
)> {
    use crate::schema::class::dsl as class;
    use crate::schema::class_student::dsl as class_student;
    use crate::schema::class_synchronous_task::dsl as class_synchronous_task;
    use crate::schema::class_teacher::dsl as class_teacher;
    use crate::schema::users::dsl as users;

    match conn
        .run(move |c| {
            class_synchronous_task::class_synchronous_task
                .inner_join(
                    class::class.inner_join(class_teacher::class_teacher.inner_join(users::users)),
                )
                .filter(users::id.eq(user_id))
                .filter(class::id.eq(class_id))
                .filter(class_synchronous_task::id.eq(task_id))
                .select(crate::schema::class_synchronous_task::all_columns)
                .first::<ClassSynchronousTask>(c)
        })
        .await
    {
        Ok(class_task) => {
            let class_task_cloned = class_task.clone();
            match conn
                .run(move |c| {
                    StudentClassSynchronousTask::belonging_to(&class_task_cloned)
                        .inner_join(class_student::class_student.inner_join(users::users))
                        .select((
                            crate::schema::users::all_columns,
                            crate::schema::student_class_synchronous_task::all_columns,
                        ))
                        .load::<(User, StudentClassSynchronousTask)>(c)
                })
                .await
            {
                Ok(tasks) => Ok((class_task, tasks)),
                Err(e) => {
                    error!("{:#?}", e);
                    Err(LovelaceError::DatabaseError)
                }
            }
        }
        Err(e) => {
            error!("{:#?}", e);
            Err(LovelaceError::DatabaseError)
        }
    }
}

struct RenderTeacherSummary(
    pub  (
        ClassSynchronousTask,
        Vec<(user::User, sync_task::StudentClassSynchronousTask)>,
    ),
);

impl Render<Html> for RenderTeacherSummary {
    fn render(self) -> Html {
        let (class_task, tasks) = self.0;
        Html::new()
            .head(default_head(format!("Task {}", class_task.title)))
            .body(
                Body::new()
                    .child(H1::new(format!("Task {}", class_task.title)))
                    .child(P::with_text(format!(
                        "Description: {}",
                        class_task.description
                    )))
                    .child(Level::new().children(tasks.into_iter().map(|(user, _)| {
                        Div::new().child(H3::new(format!("Student: {}", user.username)))
                    }))),
            )
    }
}

#[get("/<class_id>/task/sync/<task_id>/view")]
/// Retrieve information about a specific synchronous task.
pub async fn html_view_specific_synchronous_task(
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
            let summary =
                match get_teacher_sync_task_summary(task_id, class_id, auth.0, &conn).await {
                    Ok(t) => t,
                    Err(e) => return e.render(),
                };
            RenderTeacherSummary(summary).render()
        }
        crate::class::ClassMemberRole::Student => {
            let task = match get_student_sync_task_summary(task_id, class_id, auth.0, &conn).await {
                Ok(t) => t,
                Err(e) => return e.render(),
            };
            RenderStudentSummary(task).render()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudentTask {
    user: User,
    task: StudentClassSynchronousTask,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeacherViewSyncTaskRes {
    task: ClassSynchronousTask,
    student_tasks: Vec<StudentTask>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudentViewSyncTaskRes {
    task: ClassSynchronousTask,
    student_task: StudentClassSynchronousTask,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ViewSpecificSyncTaskApiRes {
    Teacher(TeacherViewSyncTaskRes),
    Student(StudentViewSyncTaskRes),
}

#[get("/<class_id>/task/sync/<task_id>/view")]
/// Retrieve information about a specific synchronous task.
pub async fn api_view_specific_synchronous_task(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Json<ApiResponse<ViewSpecificSyncTaskApiRes>> {
    let role = if let Some(role) = get_user_role_in_class(auth.0, class_id, &conn).await {
        role
    } else {
        return Json(From::from(LovelaceError::PermissionError));
    };
    Json(match role {
        crate::class::ClassMemberRole::Teacher => {
            let (task, student_tasks) =
                match get_teacher_sync_task_summary(task_id, class_id, auth.0, &conn).await {
                    Ok(t) => t,
                    Err(e) => return Json(From::from(e)),
                };
            ApiResponse::new_ok(ViewSpecificSyncTaskApiRes::Teacher(
                TeacherViewSyncTaskRes {
                    task,
                    student_tasks: student_tasks
                        .into_iter()
                        .map(|(user, task)| StudentTask { user, task })
                        .collect(),
                },
            ))
        }
        crate::class::ClassMemberRole::Student => {
            let (task, student_task) =
                match get_student_sync_task_summary(task_id, class_id, auth.0, &conn).await {
                    Ok(t) => t,
                    Err(e) => return Json(From::from(e)),
                };
            ApiResponse::new_ok(ViewSpecificSyncTaskApiRes::Student(
                StudentViewSyncTaskRes { task, student_task },
            ))
        }
    })
}
