/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
//! Synchronous tasks (e.g. homework).

use crate::{
    auth::AuthCookie,
    catch_database_error,
    class::{get_user_role_in_class, user_is_teacher, ClassMemberRole},
    css_names::{LIST, LIST_ITEM},
    db::Database,
    models::{
        ClassSynchronousTask, NewClassSynchronousTask, NewStudentClassSynchronousTask,
        StudentClassSynchronousTask, User,
    },
    utils::{
        default_head,
        error_messages::{database_error, invalid_date},
        permission_error::permission_error,
    },
};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use malvolio::prelude::*;
use rocket::FromForm;

/// Create a new form containing the necessary fields to create a new synchronous task.
fn create_new_sync_task_form() -> Form {
    Form::new()
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
/// Rust synchronous runtime. It isn't! This is just the form data supplied to the route which is
/// mounted at `class/<class_id>/task/sync/create`
pub struct CreateNewSyncTask {
    title: String,
    description: String,
    start_time: String,
    end_time: String,
}

#[get("/<class_id>/task/sync/create")]
pub async fn get_create_new_sync_task(class_id: i32, auth: AuthCookie, conn: Database) -> Html {
    if conn
        .run(move |c| user_is_teacher(auth.0, class_id, c))
        .await
    {
        Html::new().body(
            Body::new()
                .child(H1::new("Create a new synchronous task."))
                .child(create_new_sync_task_form()),
        )
    } else {
        permission_error()
    }
}

#[post("/<class_id>/task/sync/create", data = "<form>")]
pub async fn create_new_sync_task(
    conn: Database,
    class_id: i32,
    auth: AuthCookie,
    form: rocket::request::Form<CreateNewSyncTask>,
) -> Html {
    use crate::schema::class_teacher::dsl as class_teacher;
    match get_user_role_in_class(auth.0, class_id, &conn).await {
        Some(crate::class::ClassMemberRole::Teacher) => {}
        None | Some(crate::class::ClassMemberRole::Student) => return permission_error(),
    }
    let start_time = match NaiveDateTime::parse_from_str(&form.start_time, "%Y-%m-%dT%H:%M") {
        Ok(date) => date,
        Err(_) => return invalid_date(Some(create_new_sync_task_form())),
    };
    let end_time = match NaiveDateTime::parse_from_str(&form.end_time, "%Y-%m-%dT%H:%M") {
        Ok(date) => date,
        Err(_) => return invalid_date(Some(create_new_sync_task_form())),
    };
    match conn
        .run(move |c| {
            diesel::insert_into(crate::schema::class_synchronous_task::table)
                .values(NewClassSynchronousTask {
                    title: &form.title,
                    description: &form.description,
                    created: chrono::Utc::now().naive_utc(),
                    start_time,
                    end_time,
                    class_teacher_id: class_teacher::class_teacher
                        .filter(class_teacher::user_id.eq(auth.0))
                        .select(class_teacher::id)
                        .first::<i32>(c)
                        .unwrap(),
                    class_id,
                })
                .returning(crate::schema::class_synchronous_task::id)
                .get_result::<i32>(c)
        })
        .await
    {
        Ok(sync_task_id) => {
            let student_list = catch_database_error!(
                conn.run(move |c| crate::schema::class_student::table
                    .filter(crate::schema::class_student::class_id.eq(class_id))
                    .select(crate::schema::class_student::id)
                    .get_results::<i32>(c))
                    .await
            );
            match conn
                .run(move |c| {
                    diesel::insert_into(crate::schema::student_class_synchronous_task::table)
                        .values(
                            student_list
                                .into_iter()
                                .map(|class_student_id| NewStudentClassSynchronousTask {
                                    class_student_id,
                                    class_synchronous_task_id: sync_task_id,
                                })
                                .collect::<Vec<NewStudentClassSynchronousTask>>(),
                        )
                        .execute(c)
                })
                .await
            {
                Ok(_) => Html::new()
                    .head(default_head("Created that task".to_string()))
                    .body(
                        Body::new()
                            .child(H1::new("Created that task"))
                            .child(P::with_text("That task has now been sucessfully created.")),
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

/// Show a list of all the tasks in a class that a student has been assigned.
async fn show_student_sync_tasks_summary(class_id: i32, user_id: i32, conn: &Database) -> Html {
    use crate::schema::class_student::dsl as class_student;
    use crate::schema::class_synchronous_task::dsl as class_synchronous_task;
    use crate::schema::student_class_synchronous_task::dsl as STUDENT_1_class_synchronous_task;

    match conn
        .run(move |c| {
            STUDENT_1_class_synchronous_task::student_class_synchronous_task
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
        Ok(tasks) => {
            if tasks.is_empty() {
                Html::new().head(default_head("".to_string())).body(
                    Body::new().child(H1::new("Tasks for this class")).child(
                        Div::new()
                            .attribute(Class::from(LIST))
                            .children(tasks.into_iter().map(|(_, class_task_instance)| {
                                Div::new()
                                    .child(H3::new(format!("Task: {}", class_task_instance.title)))
                                    .child(P::with_text(format!(
                                        "Description: {}",
                                        class_task_instance.description
                                    )))
                            })),
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
async fn show_teacher_sync_tasks_summary(class_id: i32, conn: &Database) -> Html {
    use crate::schema::class_synchronous_task::dsl as class_synchronous_task;
    use crate::schema::class_teacher::dsl as class_teacher;
    use crate::schema::student_class_synchronous_task::dsl as student_class_synchronous_task;
    let query = class_synchronous_task::class_synchronous_task
        .filter(class_synchronous_task::class_id.eq(class_id))
        .order_by(class_synchronous_task::start_time.desc())
        .inner_join(student_class_synchronous_task::student_class_synchronous_task);
    let tasks = catch_database_error!(
        conn.run(move |c| query
            .inner_join(class_teacher::class_teacher.inner_join(crate::schema::users::dsl::users))
            .select((
                crate::schema::class_synchronous_task::all_columns,
                crate::schema::users::all_columns,
            ))
            .load::<(ClassSynchronousTask, User)>(c))
            .await
    );
    Html::new().head(default_head("Tasks".to_string())).body(
        Body::new().child(
            Div::new()
                .attribute(Class::from(LIST))
                .children(tasks.into_iter().map(|(task, set_by)| {
                    Div::new()
                        .attribute(Class::from(LIST_ITEM))
                        .child(task.render())
                        .child(P::with_text(format!("Set by: {}", set_by.username)))
                })),
        ),
    )
}

#[get("/<class_id>/task/sync/all")]
/// Show a list of all the synchronous tasks have been set in a class, either to a teacher or a
/// student (this is retrieved from the database).
pub async fn view_all_sync_tasks_in_class(class_id: i32, auth: AuthCookie, conn: Database) -> Html {
    if let Some(role) = get_user_role_in_class(auth.0, class_id, &conn).await {
        match role {
            crate::class::ClassMemberRole::Teacher => {
                show_teacher_sync_tasks_summary(class_id, &conn).await
            }
            crate::class::ClassMemberRole::Student => {
                show_student_sync_tasks_summary(class_id, auth.0, &conn).await
            }
        }
    } else {
        permission_error()
    }
}

async fn show_student_sync_task_summary(
    task_id: i32,
    class_id: i32,
    user_id: i32,
    conn: &Database,
) -> Html {
    use crate::schema::class_student::dsl as class_student;
    use crate::schema::class_synchronous_task::dsl as class_synchronous_task;
    match conn
        .run(move |c| {
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
    {
        Ok((class_task, _)) => Html::new().head(default_head("Task".to_string())).body(
            Body::new()
                .child(H1::new(format!("Task {}", class_task.title)))
                .child(P::with_text(format!(
                    "Description {}",
                    class_task.description
                ))),
        ),
        Err(e) => {
            error!("{:#?}", e);
            database_error()
        }
    }
}

async fn show_teacher_sync_task_summary(
    task_id: i32,
    class_id: i32,
    user_id: i32,
    conn: &Database,
) -> Html {
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
                Ok(tasks) => Html::new()
                    .head(default_head(format!("Task {}", class_task.title)))
                    .body(
                        Body::new()
                            .child(H1::new(format!("Task {}", class_task.title)))
                            .child(P::with_text(format!(
                                "Description: {}",
                                class_task.description
                            )))
                            .child(Div::new().attribute(Class::from(LIST)).children(
                                tasks.into_iter().map(|(user, _)| {
                                    Div::new()
                                        .attribute(Class::from(LIST_ITEM))
                                        .child(H3::new(format!("Student: {}", user.username)))
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

#[get("/<class_id>/task/sync/<task_id>/view")]
/// Retrieve information about a specific synchronous task.
pub async fn view_specific_synchronous_task(
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
            show_teacher_sync_task_summary(task_id, class_id, auth.0, &conn).await
        }
        crate::class::ClassMemberRole::Student => {
            show_student_sync_task_summary(task_id, class_id, auth.0, &conn).await
        }
    }
}

fn edit_task_form(
    title: Option<String>,
    description: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
) -> Form {
    Form::new()
        .child(
            Input::new()
                .attribute(Type::Text)
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
                .map(|item| {
                    if let Some(start_time) = start_time {
                        item.attribute(Value::new(start_time))
                    } else {
                        item
                    }
                })
                .attribute(Name::new("start_time")),
        )
        .child(
            Input::new()
                .attribute(Type::DateTimeLocal)
                .map(|item| {
                    if let Some(end_time) = end_time {
                        item.attribute(Value::new(end_time))
                    } else {
                        item
                    }
                })
                .attribute(Name::new("end_time")),
        )
}

#[get("/<class_id>/task/sync/<task_id>/edit")]
pub async fn view_edit_task_page(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Html {
    use crate::schema::class_synchronous_task::dsl as class_synchronous_task;
    if let Some(role) = get_user_role_in_class(auth.0, class_id, &conn).await {
        if role != ClassMemberRole::Teacher {
            return permission_error();
        }
        let res = catch_database_error!(
            conn.run(move |c| class_synchronous_task::class_synchronous_task
                .filter(class_synchronous_task::id.eq(task_id))
                .first::<ClassSynchronousTask>(c))
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
                        Some(res.start_time.format("%Y-%m-%dT%H:%M").to_string()),
                        Some(res.end_time.format("%Y-%m-%dT%H:%M").to_string()),
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
    start_time: String,
    end_time: String,
}

#[post("/<class_id>/task/sync/<task_id>/edit", data = "<form>")]
pub async fn apply_edit_task(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
    form: rocket::request::Form<EditTaskForm>,
) -> Html {
    use crate::schema::class_synchronous_task::dsl as class_synchronous_task;
    if let Some(role) = get_user_role_in_class(auth.0, class_id, &conn).await {
        if role != ClassMemberRole::Teacher {
            return permission_error();
        }
        let end_time = match NaiveDateTime::parse_from_str(&form.end_time, "%Y-%m-%dT%H:%M") {
            Ok(date) => date,
            Err(_) => {
                return invalid_date(Some(edit_task_form(
                    Some(form.title.clone()),
                    Some(form.description.clone()),
                    Some(form.start_time.clone()),
                    Some(form.end_time.clone()),
                )))
            }
        };
        let start_time = match NaiveDateTime::parse_from_str(&form.start_time, "%Y-%m-%dT%H:%M") {
            Ok(date) => date,
            Err(_) => {
                return invalid_date(Some(edit_task_form(
                    Some(form.title.clone()),
                    Some(form.description.clone()),
                    Some(form.start_time.clone()),
                    Some(form.end_time.clone()),
                )))
            }
        };
        match conn
            .run(move |c| {
                diesel::update(
                    class_synchronous_task::class_synchronous_task
                        .filter(class_synchronous_task::id.eq(task_id))
                        .filter(class_synchronous_task::class_id.eq(class_id)),
                )
                .set((
                    class_synchronous_task::title.eq(&form.title),
                    class_synchronous_task::description.eq(&form.description),
                    class_synchronous_task::start_time.eq(start_time),
                    class_synchronous_task::end_time.eq(end_time),
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

#[get("/<class_id>/task/sync/<task_id>/delete")]
pub async fn delete_task(class_id: i32, task_id: i32, auth: AuthCookie, conn: Database) -> Html {
    use crate::schema::class_synchronous_task::dsl as class_synchronous_task;
    if let Some(ClassMemberRole::Teacher) = get_user_role_in_class(auth.0, class_id, &conn).await {
        catch_database_error!(
            conn.run(move |c| diesel::delete(
                class_synchronous_task::class_synchronous_task
                    .filter(class_synchronous_task::id.eq(task_id))
                    .filter(class_synchronous_task::class_id.eq(class_id)),
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
mod synchronous_task_tests {
    use std::ops::Add;

    use crate::{
        db::{Database, DatabaseConnection},
        models::{
            ClassSynchronousTask, NewClassStudent, NewClassSynchronousTask, NewClassTeacher,
            NewStudentClassSynchronousTask, StudentClassSynchronousTask,
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
        let task_1_id = diesel::insert_into(crate::schema::class_synchronous_task::table)
            .values(NewClassSynchronousTask {
                title: TASK_1_TITLE,
                description: TASK_1_DESCRIPTION,
                created: chrono::Utc::now().naive_utc(),
                start_time: chrono::Utc::now()
                    .add(chrono::Duration::days(3))
                    .naive_utc(),
                end_time: chrono::Utc::now()
                    .add(chrono::Duration::days(4))
                    .naive_utc(),
                class_teacher_id,
                class_id,
            })
            .returning(crate::schema::class_synchronous_task::id)
            .get_result::<i32>(conn)
            .unwrap();
        diesel::insert_into(crate::schema::student_class_synchronous_task::table)
            .values(NewStudentClassSynchronousTask {
                class_student_id: class_student_1_id,
                class_synchronous_task_id: task_1_id,
            })
            .execute(conn)
            .unwrap();
        diesel::insert_into(crate::schema::student_class_synchronous_task::table)
            .values(NewStudentClassSynchronousTask {
                class_student_id: class_student_2_id,
                class_synchronous_task_id: task_1_id,
            })
            .execute(conn)
            .unwrap();
        let task_2_id = diesel::insert_into(crate::schema::class_synchronous_task::table)
            .values(NewClassSynchronousTask {
                title: TASK_2_TITLE,
                description: TASK_2_DESCRIPTION,
                created: chrono::Utc::now().naive_utc(),
                start_time: chrono::Utc::now()
                    .add(chrono::Duration::days(3))
                    .naive_utc(),
                end_time: chrono::Utc::now()
                    .add(chrono::Duration::days(3))
                    .naive_utc(),
                class_teacher_id,
                class_id,
            })
            .returning(crate::schema::class_synchronous_task::id)
            .get_result::<i32>(conn)
            .unwrap();
        diesel::insert_into(crate::schema::student_class_synchronous_task::table)
            .values(NewStudentClassSynchronousTask {
                class_student_id: class_student_1_id,
                class_synchronous_task_id: task_2_id,
            })
            .execute(conn)
            .unwrap();
        diesel::insert_into(crate::schema::student_class_synchronous_task::table)
            .values(NewStudentClassSynchronousTask {
                class_student_id: class_student_2_id,
                class_synchronous_task_id: task_2_id,
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
    #[tokio::test]
    async fn test_teacher_can_view_specific_synchronous_task() {
        let client = client();
        let (class_id, _, _, tasks) = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;
        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client);
        let mut view_task_res = client
            .get(format!("/class/{}/task/sync/{}/view", class_id, tasks[0]))
            .dispatch();
        let string = view_task_res.into_string().expect("invalid body response");
        assert!(string.contains(TASK_1_TITLE));
        assert!(string.contains(TASK_1_DESCRIPTION));
    }
    #[tokio::test]
    async fn test_student_can_view_specific_synchronous_task() {
        let client = client();
        let (class_id, _, _, tasks) = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;

        login_user(STUDENT_1_USERNAME, STUDENT_1_PASSWORD, &client);
        let mut view_task_res = client
            .get(format!("/class/{}/task/sync/{}/view", class_id, tasks[0]))
            .dispatch();
        let string = view_task_res.into_string().expect("invalid body response");
        assert!(string.contains(TASK_1_TITLE));
        assert!(string.contains(TASK_1_DESCRIPTION));

        login_user(STUDENT_2_USERNAME, STUDENT_2_PASSWORD, &client);
        let mut view_task_res = client
            .get(format!("/class/{}/task/sync/{}/view", class_id, tasks[0]))
            .dispatch();
        let string = view_task_res.into_string().expect("invalid body response");
        assert!(string.contains(TASK_1_TITLE));
        assert!(string.contains(TASK_1_DESCRIPTION));
    }
    #[tokio::test]
    async fn test_teacher_can_create_synchronous_task() {
        const NEW_TASK_TITLE: &str = "new-task-title";
        const NEW_TASK_DESCRIPTION: &str = "new-task-description";
        let client = client();
        let (class_id, _, _, _) = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;
        login_user(TEACHER_EMAIL, TEACHER_PASSWORD, &client);

        let mut res = client
            .post(format!("/class/{}/task/sync/create", class_id))
            .header(ContentType::Form)
            .body(format!(
                "title={}&description={}&start_time={}&end_time={}",
                NEW_TASK_TITLE,
                NEW_TASK_DESCRIPTION,
                (chrono::Utc::now() + chrono::Duration::days(7))
                    .naive_utc()
                    .format("%Y-%m-%dT%H:%M")
                    .to_string(),
                (chrono::Utc::now() + chrono::Duration::days(7))
                    .naive_utc()
                    .format("%Y-%m-%dT%H:%M")
                    .to_string()
            ))
            .dispatch();
        let string = res.into_string().expect("invalid body response");
        println!("{}", string);
        assert!(string.contains("Created that task"));
        {
            use crate::schema::class_synchronous_task::dsl as class_synchronous_task;
            use crate::schema::student_class_synchronous_task::dsl as student_class_synchronous_task;

            let results = Database::get_one(&client.rocket())
                .await
                .unwrap()
                .run(|c| {
                    class_synchronous_task::class_synchronous_task
                        .filter(class_synchronous_task::description.eq(NEW_TASK_DESCRIPTION))
                        .filter(class_synchronous_task::title.eq(NEW_TASK_TITLE))
                        .inner_join(student_class_synchronous_task::student_class_synchronous_task)
                        .load::<(ClassSynchronousTask, StudentClassSynchronousTask)>(c)
                })
                .await
                .unwrap();
            assert_eq!(results.len(), 2);
            assert_eq!(results[0].0, results[1].0);
        }
    }
    #[tokio::test]
    async fn test_teacher_can_edit_synchronous_task() {
        const NEW_TASK_TITLE: &str = "new-task-title";
        const NEW_TASK_DESCRIPTION: &str = "new-task-description";
        let client = client();
        let (class_id, _, _, tasks) = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;
        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client);
        let mut res = client
            .post(format!("/class/{}/task/sync/{}/edit", class_id, tasks[0]))
            .header(ContentType::Form)
            .body(format!(
                "title={}&description={}&start_time={}&end_time={}",
                NEW_TASK_TITLE,
                NEW_TASK_DESCRIPTION,
                (chrono::Utc::now() + chrono::Duration::days(7))
                    .naive_utc()
                    .format("%Y-%m-%dT%H:%M")
                    .to_string(),
                (chrono::Utc::now() + chrono::Duration::days(8))
                    .naive_utc()
                    .format("%Y-%m-%dT%H:%M")
                    .to_string()
            ))
            .dispatch();
        let string = res.into_string().expect("invalid body response");
        assert!(string.contains("updated that task"));
    }
    #[tokio::test]
    async fn test_student_cannot_edit_asynchronus_task() {
        const NEW_TASK_TITLE: &str = "new-task-title";
        const NEW_TASK_DESCRIPTION: &str = "new-task-description";
        let client = client();
        let (class_id, _, _, tasks) = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;
        login_user(STUDENT_1_USERNAME, STUDENT_1_PASSWORD, &client);
        let mut res = client
            .post(format!("/class/{}/task/sync/{}/edit", class_id, tasks[0]))
            .header(ContentType::Form)
            .body(format!(
                "title={}&description={}&start_time={}&end_time={}",
                NEW_TASK_TITLE,
                NEW_TASK_DESCRIPTION,
                (chrono::Utc::now() + chrono::Duration::days(7))
                    .naive_utc()
                    .format("%Y-%m-%dT%H:%M")
                    .to_string(),
                (chrono::Utc::now() + chrono::Duration::days(8))
                    .naive_utc()
                    .format("%Y-%m-%dT%H:%M")
                    .to_string()
            ))
            .dispatch();
        let string = res.into_string().expect("invalid body response");
        assert!(!string.contains("updated that task"));
    }
    #[tokio::test]
    async fn test_teacher_can_delete_synchronous_task() {
        let client = client();
        let (class_id, _, _, tasks) = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;
        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client);
        let mut res = client
            .get(format!("/class/{}/task/sync/{}/delete", class_id, tasks[1]))
            .dispatch();
        let string = res.into_string().expect("invalid body response");
        assert!(string.contains("deleted that task"));
    }
}
