use diesel::prelude::*;
use malvolio::prelude::*;

use crate::{
    css_names::LIST,
    db::Database,
    models::{ClassAsynchronousTask, StudentClassAsynchronousTask},
    utils::default_head,
};

use super::ShowAsyncTaskSummaryError;

/// Show a list of all the tasks in a class that a student has been assigned.
pub async fn get_student_async_tasks_summary(
    class_id: i32,
    user_id: i32,
    conn: &Database,
) -> Result<Vec<(StudentClassAsynchronousTask, ClassAsynchronousTask)>, ShowAsyncTaskSummaryError> {
    use crate::schema::class_asynchronous_task::dsl as class_asynchronous_task;
    use crate::schema::class_student::dsl as class_student;
    use crate::schema::student_class_asynchronous_task::dsl as student_class_asynchronous_task;

    match conn
        .run(move |c| {
            student_class_asynchronous_task::student_class_asynchronous_task
                .inner_join(class_student::class_student)
                .filter(class_student::user_id.eq(user_id))
                .inner_join(class_asynchronous_task::class_asynchronous_task)
                .filter(class_asynchronous_task::class_id.eq(class_id))
                .select((
                    crate::schema::student_class_asynchronous_task::all_columns,
                    crate::schema::class_asynchronous_task::all_columns,
                ))
                .load::<(StudentClassAsynchronousTask, ClassAsynchronousTask)>(c)
        })
        .await
    {
        Ok(tasks) => Ok(tasks),
        Err(e) => {
            error!("{:#?}", e);
            Err(ShowAsyncTaskSummaryError::DatabaseError)
        }
    }
}

pub fn render_student_task_summary(
    tasks: Vec<(StudentClassAsynchronousTask, ClassAsynchronousTask)>,
) -> Html {
    if !tasks.is_empty() {
        Html::new().head(default_head("".to_string())).body(
            Body::new().child(H1::new("Tasks for this class")).child(
                Div::new()
                    .attribute(Class::from(LIST))
                    .children(tasks.into_iter().map(
                        |(student_task_instance, class_task_instance)| {
                            Div::new()
                                .child(H3::new(format!("Task: {}", class_task_instance.title)))
                                .child(P::with_text(format!(
                                    "Description: {}",
                                    class_task_instance.description
                                )))
                                .child(P::with_text(format!(
                                    "Completed: {}",
                                    student_task_instance.completed
                                )))
                        },
                    )),
            ),
        )
    } else {
        Html::new()
            .head(default_head("No tasks found.".to_string()))
            .body(Body::new().child(H1::new("No tasks have been set for this class yet.")))
    }
}
