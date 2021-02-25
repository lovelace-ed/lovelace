use super::ShowAsyncTaskSummaryError;
use crate::{
    db::Database,
    models::{ClassAsynchronousTask, User},
    utils::default_head,
};

use diesel::prelude::*;
use malvolio::prelude::*;
use portia::levels::Level;

/// Show the list of tasks that have been set in a class. At some point we'll want to add pagination
/// support for this.
///
/// MAKE SURE YOU HAVE CHECKED THAT THE USER IS A TEACHER IN THE CLASS BEFORE YOU CALL THIS
/// FUNCTION. (sorry for the all caps, I (@teymour-aldridge) kept forgetting to do so :-)
pub async fn get_teacher_async_tasks_summary(
    class_id: i32,
    _user_id: i32,
    conn: &Database,
) -> Result<(Vec<(ClassAsynchronousTask, User)>, Vec<i64>, i64), ShowAsyncTaskSummaryError> {
    use crate::schema::class_asynchronous_task::dsl as class_asynchronous_task;
    use crate::schema::class_teacher::dsl as class_teacher;
    use crate::schema::student_class_asynchronous_task::dsl as STUDENT_1_class_asynchronous_task;
    let query = class_asynchronous_task::class_asynchronous_task
        .filter(class_asynchronous_task::class_id.eq(class_id))
        // tasks due most recently first
        .order_by(class_asynchronous_task::due_date.desc())
        .inner_join(STUDENT_1_class_asynchronous_task::student_class_asynchronous_task);
    let tasks = conn
        .run(move |c| {
            query
                .inner_join(
                    class_teacher::class_teacher.inner_join(crate::schema::users::dsl::users),
                )
                .select((
                    crate::schema::class_asynchronous_task::all_columns,
                    crate::schema::users::all_columns,
                ))
                .load::<(ClassAsynchronousTask, User)>(c)
        })
        .await
        .map_err(|e| {
            error!("{:#?}", e);
            ShowAsyncTaskSummaryError::DatabaseError
        })?;
    let completion_count = conn
        .run(move |c| {
            query
                .select(diesel::dsl::count(
                    STUDENT_1_class_asynchronous_task::completed.eq(true),
                ))
                .get_results::<i64>(c)
        })
        .await
        .map_err(|e| {
            error!("{:#?}", e);
            ShowAsyncTaskSummaryError::DatabaseError
        })?;
    let student_count = crate::models::Class::student_count(class_id, &conn)
        .await
        .map_err(|e| {
            error!("{:#?}", e);
            ShowAsyncTaskSummaryError::DatabaseError
        })?;
    Ok((tasks, completion_count, student_count))
}

pub fn render_teacher_async_tasks(
    tasks: Vec<(ClassAsynchronousTask, User)>,
    completion_count: Vec<i64>,
    student_count: i64,
) -> Html {
    Html::new()
        .head(default_head("Tasks".to_string()))
        .body(Body::new().child(
            Level::new().children(tasks.into_iter().zip(completion_count).map(
                |((task, set_by), completed_count)| {
                    Div::new()
                        .child(task.render())
                        .child(P::with_text(format!("Set by: {}", set_by.username)))
                        .child(P::with_text(format!(
                            "{} out of {} students have marked this task as complete",
                            completed_count, student_count
                        )))
                },
            )),
        ))
}
