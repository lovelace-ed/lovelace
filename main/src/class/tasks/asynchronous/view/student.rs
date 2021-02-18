use diesel::prelude::*;
use malvolio::prelude::*;

use crate::{
    db::Database,
    models::{ClassAsynchronousTask, StudentClassAsynchronousTask},
    utils::default_head,
};

use super::ViewAsyncTaskSummaryError;

pub async fn get_student_async_task_summary(
    task_id: i32,
    class_id: i32,
    user_id: i32,
    conn: &Database,
) -> Result<(ClassAsynchronousTask, StudentClassAsynchronousTask), ViewAsyncTaskSummaryError> {
    use crate::schema::class_asynchronous_task::dsl as class_asynchronous_task;
    use crate::schema::class_student::dsl as class_student;
    conn.run(move |c| {
        crate::schema::student_class_asynchronous_task::table
            .inner_join(class_asynchronous_task::class_asynchronous_task)
            .filter(class_asynchronous_task::id.eq(task_id))
            .filter(class_asynchronous_task::class_id.eq(class_id))
            .inner_join(class_student::class_student)
            .filter(class_student::user_id.eq(user_id))
            .filter(class_student::class_id.eq(class_id))
            .select((
                crate::schema::class_asynchronous_task::all_columns,
                crate::schema::student_class_asynchronous_task::all_columns,
            ))
            .first::<(ClassAsynchronousTask, StudentClassAsynchronousTask)>(c)
    })
    .await
    .map_err(|e| {
        error!("{:#?}", e);
        ViewAsyncTaskSummaryError::DatabaseError
    })
}

pub fn render_student_task_summary(
    class_task: ClassAsynchronousTask,
    student_task: StudentClassAsynchronousTask,
) -> Html {
    Html::new().head(default_head("Task".to_string())).body(
        Body::new()
            .child(H1::new(format!("Task {}", class_task.title)))
            .child(P::with_text(format!(
                "Description {}",
                class_task.description
            )))
            .child(P::with_text(if !student_task.completed {
                "You have not marked this task as done"
            } else {
                "You have marked this task as done."
            })),
    )
}
