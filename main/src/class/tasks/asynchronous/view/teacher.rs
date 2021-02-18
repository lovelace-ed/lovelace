use crate::rocket::futures::TryFutureExt;
use diesel::prelude::*;
use malvolio::prelude::*;

use crate::{
    css_names::{LIST, LIST_ITEM},
    db::Database,
    models::{ClassAsynchronousTask, StudentClassAsynchronousTask, User},
    utils::default_head,
};

use super::ViewAsyncTaskSummaryError;

pub async fn get_teacher_async_task_summary(
    task_id: i32,
    class_id: i32,
    user_id: i32,
    conn: &Database,
) -> Result<
    (
        ClassAsynchronousTask,
        Vec<(User, StudentClassAsynchronousTask)>,
    ),
    ViewAsyncTaskSummaryError,
> {
    use crate::schema::class::dsl as class;
    use crate::schema::class_asynchronous_task::dsl as class_asynchronous_task;
    use crate::schema::class_student::dsl as class_student;
    use crate::schema::class_teacher::dsl as class_teacher;
    use crate::schema::users::dsl as users;

    conn.run(move |c| {
        class_asynchronous_task::class_asynchronous_task
            .inner_join(
                class::class.inner_join(class_teacher::class_teacher.inner_join(users::users)),
            )
            .filter(users::id.eq(user_id))
            .filter(class::id.eq(class_id))
            .filter(class_asynchronous_task::id.eq(task_id))
            .select(crate::schema::class_asynchronous_task::all_columns)
            .first::<ClassAsynchronousTask>(c)
    })
    .and_then(|class_task| async move {
        let cloned_class_task = class_task.clone();
        conn.run(move |c| {
            StudentClassAsynchronousTask::belonging_to(&cloned_class_task)
                .inner_join(class_student::class_student.inner_join(users::users))
                .select((
                    crate::schema::users::all_columns,
                    crate::schema::student_class_asynchronous_task::all_columns,
                ))
                .load::<(User, StudentClassAsynchronousTask)>(c)
        })
        .await
        .map(|res| (class_task, res))
    })
    .await
    .map_err(|e| {
        error!("{:#?}", e);
        ViewAsyncTaskSummaryError::DatabaseError
    })
}

pub fn render_teacher_task_summary(
    class_task: ClassAsynchronousTask,
    tasks: Vec<(User, StudentClassAsynchronousTask)>,
) -> Html {
    Html::new()
        .head(default_head(format!("Task {}", class_task.title)))
        .body(
            Body::new()
                .child(H1::new(format!("Task {}", class_task.title)))
                .child(P::with_text(format!(
                    "Description: {}",
                    class_task.description
                )))
                .child(P::with_text(format!(
                    "{} of {} completed",
                    tasks
                        .iter()
                        .map(|(_, task)| if task.completed { 1 } else { 0 })
                        .sum::<i32>(),
                    tasks.len()
                )))
                .child(
                    Div::new()
                        .attribute(Class::from(LIST))
                        .children(tasks.into_iter().map(|(user, task)| {
                            Div::new()
                                .attribute(Class::from(LIST_ITEM))
                                .child(H3::new(format!("Student: {}", user.username)))
                                .child(P::with_text(format!("Completed: {}", task.completed)))
                        })),
                ),
        )
}
