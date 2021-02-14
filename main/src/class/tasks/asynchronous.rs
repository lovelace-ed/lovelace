/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
//! Asynchronous tasks (e.g. homework).

use crate::{
    auth::AuthCookie,
    calendar::scheduler::schedule_class,
    catch_database_error,
    class::{get_user_role_in_class, user_is_teacher, ClassMemberRole},
    css_names::{LIST, LIST_ITEM},
    db::Database,
    models::{
        ClassAsynchronousTask, NewClassAsynchronousTask, NewStudentClassAsynchronousTask,
        StudentClassAsynchronousTask, User,
    },
    utils::{
        default_head,
        error_messages::{database_error, invalid_date},
        permission_error::permission_error,
    },
};
use chrono::{Duration, NaiveDateTime, Utc};
use diesel::prelude::*;
use malvolio::prelude::*;
use mercutio::Apply;
use portia::form::{FormStyle, FormSubmitInputStyle, FormTextInputStyle};
use rocket::FromForm;

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

#[derive(FromForm, Debug, Clone)]
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

#[post("/<class_id>/task/async/create", data = "<form>")]
pub async fn create_new_async_task(
    conn: Database,
    class_id: i32,
    auth: AuthCookie,
    form: rocket::request::Form<CreateNewAsyncTask>,
) -> Html {
    use crate::schema::class_teacher::dsl as class_teacher;
    match get_user_role_in_class(auth.0, class_id, &conn).await {
        Some(crate::class::ClassMemberRole::Teacher) => {}
        None | Some(crate::class::ClassMemberRole::Student) => return permission_error(),
    };
    let due_date = match NaiveDateTime::parse_from_str(&form.due_date, "%Y-%m-%dT%H:%M") {
        Ok(date) => date,
        Err(_) => return invalid_date(Some(create_new_async_task_form())),
    };
    match conn
        .run(move |c| {
            diesel::insert_into(crate::schema::class_asynchronous_task::table)
                .values(NewClassAsynchronousTask {
                    title: &form.title,
                    description: &form.description,
                    created: chrono::Utc::now().naive_utc(),
                    due_date,
                    class_teacher_id: class_teacher::class_teacher
                        .filter(class_teacher::user_id.eq(auth.0))
                        .select(class_teacher::id)
                        .first::<i32>(c)
                        .unwrap(),
                    class_id,
                })
                .returning(crate::schema::class_asynchronous_task::id)
                .get_result::<i32>(c)
        })
        .await
    {
        Ok(async_task_id) => {
            let student_list = catch_database_error!(
                conn.run(move |c| crate::schema::class_student::table
                    .filter(crate::schema::class_student::class_id.eq(class_id))
                    .select(crate::schema::class_student::id)
                    .get_results::<i32>(c))
                    .await
            );
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
                    Html::new()
                        .head(default_head("Created that task".to_string()))
                        .body(
                            Body::new()
                                .child(H1::new("Created that task"))
                                .child(P::with_text("That task has now been sucessfully created.")),
                        )
                }
                Err(e) => {
                    error!("{:#?}", e);
                    database_error()
                }
            }
        }
        Err(e) => {
            error!("{:#?}", e);
            database_error()
        }
    }
}

/// Show a list of all the tasks in a class that a student has been assigned.
async fn show_student_async_tasks_summary(class_id: i32, user_id: i32, conn: &Database) -> Html {
    use crate::schema::class_asynchronous_task::dsl as class_asynchronous_task;
    use crate::schema::class_student::dsl as class_student;
    use crate::schema::student_class_asynchronous_task::dsl as STUDENT_1_class_asynchronous_task;

    match conn
        .run(move |c| {
            STUDENT_1_class_asynchronous_task::student_class_asynchronous_task
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
        Ok(tasks) => {
            if tasks.is_empty() {
                Html::new().head(default_head("".to_string())).body(
                    Body::new().child(H1::new("Tasks for this class")).child(
                        Div::new()
                            .attribute(Class::from(LIST))
                            .children(tasks.into_iter().map(
                                |(student_task_instance, class_task_instance)| {
                                    Div::new()
                                        .child(H3::new(format!(
                                            "Task: {}",
                                            class_task_instance.title
                                        )))
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
        Err(e) => {
            error!("{:#?}", e);
            database_error()
        }
    }
}

/// Show the list of tasks that have been set in a class. At some point we'll want to add pagination
/// support for this.
///
/// MAKE SURE YOU HAVE CHECKED THAT THE USER IS A TEACHER IN THE CLASS BEFORE YOU CALL THIS
/// FUNCTION. (sorry for the all caps, I (@teymour-aldridge) kept forgetting to do so :-)
async fn show_teacher_async_tasks_summary(class_id: i32, conn: &Database) -> Html {
    use crate::schema::class_asynchronous_task::dsl as class_asynchronous_task;
    use crate::schema::class_teacher::dsl as class_teacher;
    use crate::schema::student_class_asynchronous_task::dsl as STUDENT_1_class_asynchronous_task;
    let query = class_asynchronous_task::class_asynchronous_task
        .filter(class_asynchronous_task::class_id.eq(class_id))
        // tasks due most recently first
        .order_by(class_asynchronous_task::due_date.desc())
        .inner_join(STUDENT_1_class_asynchronous_task::student_class_asynchronous_task);
    let tasks = catch_database_error!(
        conn.run(move |c| query
            .inner_join(class_teacher::class_teacher.inner_join(crate::schema::users::dsl::users))
            .select((
                crate::schema::class_asynchronous_task::all_columns,
                crate::schema::users::all_columns,
            ))
            .load::<(ClassAsynchronousTask, User)>(c))
            .await
    );
    let completion_count = catch_database_error!(
        conn.run(move |c| query
            .select(diesel::dsl::count(
                STUDENT_1_class_asynchronous_task::completed.eq(true),
            ))
            .get_results::<i64>(c))
            .await
    );
    let student_count =
        catch_database_error!(crate::models::Class::student_count(class_id, &conn).await);
    Html::new().head(default_head("Tasks".to_string())).body(
        Body::new().child(
            Div::new().attribute(Class::from(LIST)).children(
                tasks
                    .into_iter()
                    .zip(completion_count)
                    .map(|((task, set_by), completed_count)| {
                        Div::new()
                            .attribute(Class::from(LIST_ITEM))
                            .child(task.render())
                            .child(P::with_text(format!("Set by: {}", set_by.username)))
                            .child(P::with_text(format!(
                                "{} out of {} students have marked this task as complete",
                                completed_count, student_count
                            )))
                    }),
            ),
        ),
    )
}

#[get("/<class_id>/task/async/all")]
/// Show a list of all the asynchronous tasks have been set in a class, either to a teacher or a
/// student (this is retrieved from the database).
pub async fn view_all_async_tasks_in_class(
    class_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Html {
    if let Some(role) = get_user_role_in_class(auth.0, class_id, &conn).await {
        match role {
            crate::class::ClassMemberRole::Teacher => {
                show_teacher_async_tasks_summary(class_id, &conn).await
            }
            crate::class::ClassMemberRole::Student => {
                show_student_async_tasks_summary(class_id, auth.0, &conn).await
            }
        }
    } else {
        permission_error()
    }
}

async fn show_student_async_task_summary(
    task_id: i32,
    class_id: i32,
    user_id: i32,
    conn: &Database,
) -> Html {
    use crate::schema::class_asynchronous_task::dsl as class_asynchronous_task;
    use crate::schema::class_student::dsl as class_student;
    match conn
        .run(move |c| {
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
    {
        Ok((class_task, student_task)) => Html::new().head(default_head("Task".to_string())).body(
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
        ),
        Err(e) => {
            error!("{:#?}", e);
            database_error()
        }
    }
}

async fn show_teacher_async_task_summary(
    task_id: i32,
    class_id: i32,
    user_id: i32,
    conn: &Database,
) -> Html {
    use crate::schema::class::dsl as class;
    use crate::schema::class_asynchronous_task::dsl as class_asynchronous_task;
    use crate::schema::class_student::dsl as class_student;
    use crate::schema::class_teacher::dsl as class_teacher;
    use crate::schema::users::dsl as users;

    match conn
        .run(move |c| {
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
        .await
    {
        Ok(class_task) => {
            let cloned_class_task = class_task.clone();
            match conn
                .run(move |c| {
                    StudentClassAsynchronousTask::belonging_to(&cloned_class_task)
                        .inner_join(class_student::class_student.inner_join(users::users))
                        .select((
                            crate::schema::users::all_columns,
                            crate::schema::student_class_asynchronous_task::all_columns,
                        ))
                        .load::<(User, StudentClassAsynchronousTask)>(c)
                })
                .await
            {
                Ok(tasks) => Html::new()
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
                            .child(Div::new().attribute(Class::from(LIST)).children(
                                tasks.into_iter().map(|(user, task)| {
                                    Div::new()
                                        .attribute(Class::from(LIST_ITEM))
                                        .child(H3::new(format!("Student: {}", user.username)))
                                        .child(P::with_text(format!(
                                            "Completed: {}",
                                            task.completed
                                        )))
                                }),
                            )),
                    ),
                Err(e) => {
                    error!("{:#?}", e);
                    database_error()
                }
            }
        }
        Err(e) => {
            error!("{:#?}", e);
            database_error()
        }
    }
}

#[get("/<class_id>/task/async/<task_id>/view")]
/// Retrieve information about a specific asynchronous task.
pub async fn view_specific_asynchronous_task(
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
            show_teacher_async_task_summary(task_id, class_id, auth.0, &conn).await
        }
        crate::class::ClassMemberRole::Student => {
            show_student_async_task_summary(task_id, class_id, auth.0, &conn).await
        }
    }
}

fn edit_task_form(
    title: Option<String>,
    description: Option<String>,
    due_date: Option<String>,
) -> Form {
    Form::new()
        .apply(FormStyle)
        .child(
            Input::new()
                .attribute(Type::Text)
                .apply(FormTextInputStyle)
                .map(|item| {
                    if let Some(title) = title {
                        item.attribute(Value::new(title))
                    } else {
                        item
                    }
                })
                .attribute(Name::new("title")),
        )
        .child(
            Input::new()
                .attribute(Type::Text)
                .apply(FormTextInputStyle)
                .map(|item| {
                    if let Some(description) = description {
                        item.attribute(Value::new(description))
                    } else {
                        item
                    }
                })
                .attribute(Name::new("description")),
        )
        .child(
            Input::new()
                .attribute(Type::DateTimeLocal)
                .apply(FormSubmitInputStyle)
                .map(|item| {
                    if let Some(due_date) = due_date {
                        item.attribute(Value::new(due_date))
                    } else {
                        item
                    }
                })
                .attribute(Name::new("due_date")),
        )
}

#[get("/<class_id>/task/async/<task_id>/edit")]
pub async fn view_edit_task_page(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Html {
    use crate::schema::class_asynchronous_task::dsl as class_asynchronous_task;
    if let Some(role) = get_user_role_in_class(auth.0, class_id, &conn).await {
        if role != ClassMemberRole::Teacher {
            return permission_error();
        }
        let res = catch_database_error!(
            conn.run(move |c| class_asynchronous_task::class_asynchronous_task
                .filter(class_asynchronous_task::id.eq(task_id))
                .first::<ClassAsynchronousTask>(c))
                .await
        );
        Html::new()
            .head(default_head("Edit a task".to_string()))
            .body(
                Body::new()
                    .child(H1::new("Edit this task"))
                    .child(edit_task_form(
                        Some(res.title),
                        Some(res.description),
                        Some(res.due_date.format("%Y-%m-%dT%H:%M").to_string()),
                    )),
            )
    } else {
        permission_error()
    }
}

#[derive(FromForm, Debug, Clone)]
pub struct EditTaskForm {
    title: String,
    description: String,
    due_date: String,
}

#[post("/<class_id>/task/async/<task_id>/edit", data = "<form>")]
pub async fn apply_edit_task(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
    form: rocket::request::Form<EditTaskForm>,
) -> Html {
    use crate::schema::class_asynchronous_task::dsl as class_asynchronous_task;
    let due_date = match NaiveDateTime::parse_from_str(&form.due_date, "%Y-%m-%dT%H:%M") {
        Ok(date) => date,
        Err(_) => {
            return invalid_date(Some(edit_task_form(
                Some(form.title.clone()),
                Some(form.description.clone()),
                Some(form.due_date.clone()),
            )))
        }
    };
    if let Some(role) = get_user_role_in_class(auth.0, class_id, &conn).await {
        if role != ClassMemberRole::Teacher {
            return permission_error();
        }
        match conn
            .run(move |c| {
                diesel::update(
                    class_asynchronous_task::class_asynchronous_task
                        .filter(class_asynchronous_task::id.eq(task_id))
                        .filter(class_asynchronous_task::class_id.eq(class_id)),
                )
                .set((
                    class_asynchronous_task::title.eq(&form.title),
                    class_asynchronous_task::description.eq(&form.description),
                    class_asynchronous_task::due_date.eq(due_date),
                ))
                .execute(c)
            })
            .await
        {
            Ok(_) => Html::new()
                .head(default_head("Successfully updated".to_string()))
                .body(Body::new().child(H1::new("Successfully updated that task."))),
            Err(_) => database_error(),
        }
    } else {
        permission_error()
    }
}

#[get("/<class_id>/task/async/<task_id>/delete")]
pub async fn delete_task(class_id: i32, task_id: i32, auth: AuthCookie, conn: Database) -> Html {
    use crate::schema::class_asynchronous_task::dsl as class_asynchronous_task;
    if let Some(ClassMemberRole::Teacher) = get_user_role_in_class(auth.0, class_id, &conn).await {
        catch_database_error!(
            conn.run(move |c| diesel::delete(
                class_asynchronous_task::class_asynchronous_task
                    .filter(class_asynchronous_task::id.eq(task_id))
                    .filter(class_asynchronous_task::class_id.eq(class_id)),
            )
            .execute(c))
                .await
        );
        Html::new()
            .head(default_head("Successfully deleted that task".to_string()))
            .body(Body::new().child(H1::new("Successfully deleted that task.")))
    } else {
        permission_error()
    }
}

#[cfg(test)]
mod async_task_tests {
    use std::ops::Add;

    use crate::{
        db::{Database, DatabaseConnection},
        models::{
            ClassAsynchronousTask, NewClassAsynchronousTask, NewClassStudent, NewClassTeacher,
            NewStudentClassAsynchronousTask, StudentClassAsynchronousTask,
        },
        utils::{client, login_user},
    };

    use diesel::prelude::*;
    use rocket::http::ContentType;
    const CLASS_NAME: &str = "class_name";
    const CLASS_DESCRIPTION: &str = "class_description";
    const CLASS_CODE: &str = "12345";

    const TEACHER_USERNAME: &str = "teacher-username";
    const TEACHER_EMAIL: &str = "teacher@example.com";
    const TEACHER_PASSWORD: &str = "teacher-pwd";

    const STUDENT_1_USERNAME: &str = "student-username";
    const STUDENT_1_EMAIL: &str = "student@example.com";
    const STUDENT_1_PASSWORD: &str = "student-pwd";

    const STUDENT_2_USERNAME: &str = "student-2-username";
    const STUDENT_2_EMAIL: &str = "student2@example.com";
    const STUDENT_2_PASSWORD: &str = "student-2-pwd";

    const TASK_1_TITLE: &str = "The Task Title is Title";
    const TASK_1_DESCRIPTION: &str = "The task description is the description";

    const TASK_2_TITLE: &str = "The second task title";
    const TASK_2_DESCRIPTION: &str = "The second task description";

    const TIMEZONE: &str = "Africa/Abidjan";

    /// (class id, teacher id, student id, vec<task id>)
    fn populate_database(conn: &DatabaseConnection) -> (i32, i32, i32, Vec<i32>) {
        let class_id = diesel::insert_into(crate::schema::class::table)
            .values(crate::models::NewClass {
                name: CLASS_NAME,
                description: CLASS_DESCRIPTION,
                created: chrono::Utc::now().naive_utc(),
                code: CLASS_CODE,
            })
            .returning(crate::schema::class::id)
            .get_result::<i32>(conn)
            .unwrap();
        let teacher_id = diesel::insert_into(crate::schema::users::table)
            .values(crate::models::NewUser {
                username: TEACHER_USERNAME,
                email: TEACHER_EMAIL,
                password: &bcrypt::hash(TEACHER_PASSWORD, bcrypt::DEFAULT_COST).unwrap(),
                created: chrono::Utc::now().naive_utc(),
                email_verified: true,
                timezone: TIMEZONE,
            })
            .returning(crate::schema::users::id)
            .get_result::<i32>(conn)
            .unwrap();
        let class_teacher_id = diesel::insert_into(crate::schema::class_teacher::table)
            .values(NewClassTeacher {
                user_id: teacher_id,
                class_id,
            })
            .returning(crate::schema::class_teacher::id)
            .get_result::<i32>(conn)
            .unwrap();
        let student_1_id = diesel::insert_into(crate::schema::users::table)
            .values(crate::models::NewUser {
                username: STUDENT_1_USERNAME,
                email: STUDENT_1_EMAIL,
                password: &bcrypt::hash(STUDENT_1_PASSWORD, bcrypt::DEFAULT_COST).unwrap(),
                created: chrono::Utc::now().naive_utc(),
                email_verified: true,
                timezone: TIMEZONE,
            })
            .returning(crate::schema::users::id)
            .get_result::<i32>(conn)
            .unwrap();
        let class_student_1_id = diesel::insert_into(crate::schema::class_student::table)
            .values(NewClassStudent {
                user_id: student_1_id,
                class_id,
            })
            .returning(crate::schema::class_student::dsl::id)
            .get_result::<i32>(conn)
            .unwrap();
        let student_2_id = diesel::insert_into(crate::schema::users::table)
            .values(crate::models::NewUser {
                username: STUDENT_2_USERNAME,
                email: STUDENT_2_EMAIL,
                password: &bcrypt::hash(STUDENT_2_PASSWORD, bcrypt::DEFAULT_COST).unwrap(),
                created: chrono::Utc::now().naive_utc(),
                email_verified: true,
                timezone: TIMEZONE,
            })
            .returning(crate::schema::users::id)
            .get_result::<i32>(conn)
            .unwrap();
        let class_student_2_id = diesel::insert_into(crate::schema::class_student::table)
            .values(NewClassStudent {
                user_id: student_2_id,
                class_id,
            })
            .returning(crate::schema::class_student::dsl::id)
            .get_result::<i32>(conn)
            .unwrap();
        let task_1_id = diesel::insert_into(crate::schema::class_asynchronous_task::table)
            .values(NewClassAsynchronousTask {
                title: TASK_1_TITLE,
                description: TASK_1_DESCRIPTION,
                created: chrono::Utc::now().naive_utc(),
                due_date: chrono::Utc::now()
                    .add(chrono::Duration::days(3))
                    .naive_utc(),
                class_teacher_id,
                class_id,
            })
            .returning(crate::schema::class_asynchronous_task::id)
            .get_result::<i32>(conn)
            .unwrap();
        diesel::insert_into(crate::schema::student_class_asynchronous_task::table)
            .values(NewStudentClassAsynchronousTask {
                class_student_id: class_student_1_id,
                class_asynchronous_task_id: task_1_id,
                completed: true,
            })
            .execute(conn)
            .unwrap();
        diesel::insert_into(crate::schema::student_class_asynchronous_task::table)
            .values(NewStudentClassAsynchronousTask {
                class_student_id: class_student_2_id,
                class_asynchronous_task_id: task_1_id,
                completed: true,
            })
            .execute(conn)
            .unwrap();
        let task_2_id = diesel::insert_into(crate::schema::class_asynchronous_task::table)
            .values(NewClassAsynchronousTask {
                title: TASK_2_TITLE,
                description: TASK_2_DESCRIPTION,
                created: chrono::Utc::now().naive_utc(),
                due_date: chrono::Utc::now()
                    .add(chrono::Duration::days(3))
                    .naive_utc(),
                class_teacher_id,
                class_id,
            })
            .returning(crate::schema::class_asynchronous_task::id)
            .get_result::<i32>(conn)
            .unwrap();
        diesel::insert_into(crate::schema::student_class_asynchronous_task::table)
            .values(NewStudentClassAsynchronousTask {
                class_student_id: class_student_1_id,
                class_asynchronous_task_id: task_2_id,
                completed: false,
            })
            .execute(conn)
            .unwrap();
        diesel::insert_into(crate::schema::student_class_asynchronous_task::table)
            .values(NewStudentClassAsynchronousTask {
                class_student_id: class_student_2_id,
                class_asynchronous_task_id: task_2_id,
                completed: true,
            })
            .execute(conn)
            .unwrap();
        (
            class_id,
            teacher_id,
            student_1_id,
            vec![task_1_id, task_2_id],
        )
    }
    #[rocket::async_test]
    async fn test_teacher_can_view_specific_asynchronous_task() {
        let client = client().await;
        let (class_id, _, _, tasks) = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;
        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client).await;
        let view_task_res = client
            .get(format!("/class/{}/task/async/{}/view", class_id, tasks[0]))
            .dispatch()
            .await;
        let string = view_task_res
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains(TASK_1_TITLE));
        assert!(string.contains(TASK_1_DESCRIPTION));
        assert!(string.contains("2 of 2 completed"));
    }
    #[rocket::async_test]
    async fn test_student_can_view_specific_asynchronous_task() {
        let client = client().await;
        let (class_id, _, _, tasks) = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;

        login_user(STUDENT_1_USERNAME, STUDENT_1_PASSWORD, &client).await;
        let view_task_res = client
            .get(format!("/class/{}/task/async/{}/view", class_id, tasks[0]))
            .dispatch()
            .await;
        let string = view_task_res
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains(TASK_1_TITLE));
        assert!(string.contains(TASK_1_DESCRIPTION));
        assert!(string.contains("You have marked this task as done"));
        assert!(!string.contains("1 of 1 completed"));

        login_user(STUDENT_2_USERNAME, STUDENT_2_PASSWORD, &client).await;
        let view_task_res = client
            .get(format!("/class/{}/task/async/{}/view", class_id, tasks[0]))
            .dispatch()
            .await;
        let string = view_task_res
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains(TASK_1_TITLE));
        assert!(string.contains(TASK_1_DESCRIPTION));
        assert!(string.contains("You have marked this task as done"));
        assert!(!string.contains("1 of 1 completed"));
    }
    #[rocket::async_test]
    async fn test_teacher_can_create_asynchronous_task() {
        const NEW_TASK_TITLE: &str = "new-task-title";
        const NEW_TASK_DESCRIPTION: &str = "new-task-description";
        let client = client().await;
        let (class_id, _, _, _) = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;
        login_user(TEACHER_EMAIL, TEACHER_PASSWORD, &client).await;

        let res = client
            .post(format!("/class/{}/task/async/create", class_id))
            .header(ContentType::Form)
            .body(format!(
                "title={}&description={}&due_date={}",
                NEW_TASK_TITLE,
                NEW_TASK_DESCRIPTION,
                (chrono::Utc::now() + chrono::Duration::days(7))
                    .naive_utc()
                    .format("%Y-%m-%dT%H:%M")
                    .to_string(),
            ))
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains("Created that task"));
        {
            use crate::schema::class_asynchronous_task::dsl as class_asynchronous_task;
            use crate::schema::student_class_asynchronous_task::dsl as student_class_asynchronous_task;

            let results = Database::get_one(&client.rocket())
                .await
                .unwrap()
                .run(|c| {
                    class_asynchronous_task::class_asynchronous_task
                        .filter(class_asynchronous_task::description.eq(NEW_TASK_DESCRIPTION))
                        .filter(class_asynchronous_task::title.eq(NEW_TASK_TITLE))
                        .inner_join(
                            student_class_asynchronous_task::student_class_asynchronous_task,
                        )
                        .load::<(ClassAsynchronousTask, StudentClassAsynchronousTask)>(c)
                })
                .await
                .unwrap();
            assert_eq!(results.len(), 2);
            assert_eq!(results[0].0, results[1].0);
        }
    }
    #[rocket::async_test]
    async fn test_teacher_can_edit_asynchronous_task() {
        const NEW_TASK_TITLE: &str = "new-task-title";
        const NEW_TASK_DESCRIPTION: &str = "new-task-description";
        let client = client().await;
        let (class_id, _, _, tasks) = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;
        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client).await;
        let res = client
            .post(format!("/class/{}/task/async/{}/edit", class_id, tasks[0]))
            .header(ContentType::Form)
            .body(format!(
                "title={}&description={}&due_date={}",
                NEW_TASK_TITLE,
                NEW_TASK_DESCRIPTION,
                (chrono::Utc::now() + chrono::Duration::days(7))
                    .naive_utc()
                    .format("%Y-%m-%dT%H:%M")
                    .to_string()
            ))
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains("updated that task"));
    }
    #[rocket::async_test]
    async fn test_student_cannot_edit_asynchronus_task() {
        const NEW_TASK_TITLE: &str = "new-task-title";
        const NEW_TASK_DESCRIPTION: &str = "new-task-description";
        let client = client().await;
        let (class_id, _, _, tasks) = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;
        login_user(STUDENT_1_USERNAME, STUDENT_1_PASSWORD, &client).await;
        let res = client
            .post(format!("/class/{}/task/async/{}/edit", class_id, tasks[0]))
            .header(ContentType::Form)
            .body(format!(
                "title={}&description={}&due_date={}",
                NEW_TASK_TITLE,
                NEW_TASK_DESCRIPTION,
                (chrono::Utc::now() + chrono::Duration::days(7))
                    .naive_utc()
                    .format("%Y-%m-%dT%H:%M")
                    .to_string()
            ))
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(!string.contains("updated that task"));
    }
    #[rocket::async_test]
    async fn test_teacher_can_delete_asynchronous_task() {
        let client = client().await;
        let (class_id, _, _, tasks) = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;
        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client).await;
        let res = client
            .get(format!(
                "/class/{}/task/async/{}/delete",
                class_id, tasks[1]
            ))
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains("deleted that task"));
    }
}
