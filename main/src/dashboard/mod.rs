//! The dashboard. This is designed to be actually navigable unlike certain people's softwae (cough,
//! cough Google Classroom).

use diesel::prelude::*;
use malvolio::prelude::*;
use mercutio::Apply;
use portia::{card::Card, levels::Level, render::Render};
use rocket_contrib::json::Json;

use crate::{
    auth::AuthCookie,
    db::Database,
    models::{
        ClassAsynchronousTask, ClassStudent, ClassSynchronousTask, ClassTeacher,
        StudentClassAsynchronousTask, User,
    },
    schema::{
        class, class_asynchronous_task, class_student, class_synchronous_task, class_teacher,
        student_class_asynchronous_task, users,
    },
    utils::default_head,
    utils::{error::LovelaceError, json_response::ApiResponse},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TeacherTask {
    task: ClassAsynchronousTask,
    number_set_to: i32,
    number_completed: i32,
    class: crate::models::Class,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StudentTask {
    task: ClassAsynchronousTask,
    student_task: StudentClassAsynchronousTask,
    class: crate::models::Class,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum AsynchronousTask {
    Teacher(TeacherTask),
    Student(StudentTask),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SynchronousTask {
    task: ClassSynchronousTask,
    class: crate::models::Class,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dashboard {
    sync_tasks: Vec<SynchronousTask>,
    async_tasks: Vec<AsynchronousTask>,
}

impl Dashboard {
    /// Retrieve the dashboard from the database.
    pub async fn query(auth: AuthCookie, conn: Database) -> Result<Self, diesel::result::Error> {
        conn.run(move |c| {
            let sync_tasks = class_synchronous_task::table
                .inner_join(
                    class::table
                        .left_join(
                            class_student::table.inner_join(student_class_asynchronous_task::table),
                        )
                        .left_join(class_teacher::table),
                )
                .filter(
                    class_student::user_id
                        .eq(auth.0)
                        .or(class_teacher::user_id.eq(auth.0)),
                )
                .filter(class_synchronous_task::start_time.ge(chrono::Utc::now().naive_utc()))
                .select((class_synchronous_task::all_columns, class::all_columns))
                .load::<(ClassSynchronousTask, crate::models::Class)>(c)?
                .into_iter()
                .map(|(task, class)| SynchronousTask { task, class })
                .collect::<Vec<_>>();
            let async_tasks = class_asynchronous_task::table
                .inner_join(
                    class::table
                        .left_join(
                            class_student::table.inner_join(student_class_asynchronous_task::table),
                        )
                        .left_join(class_teacher::table),
                )
                .filter(class_asynchronous_task::due_date.ge(chrono::Utc::now().naive_utc()))
                .filter(
                    class_student::user_id
                        .eq(auth.0)
                        .or(class_teacher::user_id.eq(auth.0)),
                )
                // for each teacher task we have to send another query to the database, so to this
                // is here (the number is a total guess) to keep response times reasonable.
                .limit(5)
                .load::<(
                    ClassAsynchronousTask,
                    (
                        crate::models::Class,
                        Option<(ClassStudent, StudentClassAsynchronousTask)>,
                        Option<ClassTeacher>,
                    ),
                )>(c)
                .and_then(|list| {
                    let mut output = vec![];
                    for (task, (class, class_student, class_teacher)) in list {
                        if class_student.is_none() && class_teacher.is_none() {
                            continue;
                        }
                        let is_class_student = class_student
                            .as_ref()
                            .map(|(student, _)| student.user_id == auth.0)
                            .unwrap_or(false);
                        if is_class_student {
                            if let Some((_, student_task)) = class_student {
                                output.push(AsynchronousTask::Student(StudentTask {
                                    task,
                                    student_task,
                                    class,
                                }))
                            } else {
                                unreachable!()
                            }
                        } else if class_teacher.is_some() {
                            let student_tasks = StudentClassAsynchronousTask::belonging_to(&task)
                                .inner_join(class_student::table.inner_join(users::table))
                                .select((
                                    crate::schema::users::all_columns,
                                    crate::schema::student_class_asynchronous_task::all_columns,
                                ))
                                .load::<(User, StudentClassAsynchronousTask)>(c)?;
                            output.push(AsynchronousTask::Teacher(TeacherTask {
                                task,
                                number_set_to: student_tasks.len() as i32,
                                number_completed: student_tasks
                                    .into_iter()
                                    .map(|(_, task)| if task.completed { 1 } else { 0 })
                                    .sum::<i32>(),
                                class,
                            }))
                        } else {
                            unreachable!()
                        }
                    }
                    Ok(output)
                })?;

            Ok(Self {
                sync_tasks,
                async_tasks,
            })
        })
        .await
    }
}

impl Render<Html> for Dashboard {
    fn render(self) -> Html {
        Html::new().head(default_head("Dashboard")).body(
            Body::new()
                .child(
                    Level::new()
                        .child(H1::new("Upcoming asynchronous tasks"))
                        .children(self.async_tasks.into_iter().map(|task| {
                            Level::new()
                                .child(H3::new(format!(
                                    "Task: {}",
                                    match task {
                                        AsynchronousTask::Teacher(ref teacher_task) => {
                                            teacher_task.task.title.clone()
                                        }
                                        AsynchronousTask::Student(ref student_task) => {
                                            student_task.task.title.clone()
                                        }
                                    }
                                )))
                                .child(P::with_text(format!(
                                    "Description: {}",
                                    match task {
                                        AsynchronousTask::Teacher(ref teacher) => {
                                            teacher.task.description.clone()
                                        }
                                        AsynchronousTask::Student(ref student) => {
                                            student.task.description.clone()
                                        }
                                    }
                                )))
                                .child(P::with_text(format!(
                                    "Due date: {}",
                                    match task {
                                        AsynchronousTask::Teacher(ref teacher) => {
                                            teacher.task.due_date.format("%Y-%m-%d %H:%M:%S")
                                        }
                                        AsynchronousTask::Student(ref student) => {
                                            student.task.due_date.format("%Y-%m-%d %H:%M:%S")
                                        }
                                    }
                                )))
                                .apply(|level| match task {
                                    AsynchronousTask::Teacher(ref teacher) => {
                                        level.child(P::with_text(format!(
                                            "{} of {} completed",
                                            teacher.number_completed, teacher.number_set_to
                                        )))
                                    }
                                    AsynchronousTask::Student(_) => {
                                        println!("STUDENT");
                                        level
                                    }
                                })
                                .child(A::new().text("See more").attribute(Href::new(format!(
                                    "/class/{}/async/task/{}",
                                    match task {
                                        AsynchronousTask::Teacher(ref teacher) => {
                                            teacher.class.id
                                        }
                                        AsynchronousTask::Student(ref student) => {
                                            student.class.id
                                        }
                                    },
                                    match task {
                                        AsynchronousTask::Teacher(teacher) => {
                                            teacher.task.id
                                        }
                                        AsynchronousTask::Student(student) => {
                                            student.task.id
                                        }
                                    }
                                ))))
                        })),
                )
                .child(
                    Level::new()
                        .child(H1::new("Upcoming synchronous tasks"))
                        .children(
                            self.sync_tasks
                                .into_iter()
                                .map(|task| SyncTaskCard(task).render()),
                        ),
                ),
        )
    }
}

pub struct SyncTaskCard(pub SynchronousTask);

impl Render<Div> for SyncTaskCard {
    fn render(self) -> Div {
        Card::new()
            .title(self.0.task.title)
            .contents(BodyNode::from(
                Div::new()
                    .child(P::with_text(format!(
                        "Description: {}",
                        self.0.task.description
                    )))
                    .child(P::with_text(format!(
                        "Start time: {}",
                        self.0.task.start_time.format("%Y-%m-%d %H:%M:%S")
                    )))
                    .child(P::with_text(format!(
                        "End time: {}",
                        self.0.task.end_time.format("%Y-%m-%d %H:%M:%S")
                    ))),
            ))
            .action_bar(vec![A::new()
                .attribute(Href::new(format!(
                    "/class/{}/task/sync/{}",
                    self.0.class.id, self.0.task.id
                )))
                .text("View more")])
            .render()
    }
}

#[get("/")]
pub async fn html_dashboard(conn: Database, auth: AuthCookie) -> Html {
    match Dashboard::query(auth, conn).await {
        Ok(t) => t,
        Err(e) => {
            error!("{:#?}", e);
            return LovelaceError::DatabaseError.render();
        }
    }
    .render()
}

#[get("/")]
pub async fn api_dashboard(conn: Database, auth: AuthCookie) -> Json<ApiResponse<Dashboard>> {
    Json(match Dashboard::query(auth, conn).await {
        Ok(res) => ApiResponse::new_ok(res),
        Err(e) => {
            error!("{:#?}", e);
            From::from(LovelaceError::DatabaseError)
        }
    })
}

#[cfg(test)]
mod test_dashboard {
    use std::ops::Add;

    use chrono::{Duration, Utc};
    use diesel::prelude::*;

    use crate::{
        db::Database,
        institution::test_ctx::{
            setup_env as setup_env_to_be_extended, STUDENT_PASSWORD, STUDENT_USERNAME,
            TEACHER_PASSWORD, TEACHER_USERNAME,
        },
        models::{
            NewClass, NewClassAsynchronousTask, NewClassStudent, NewClassSynchronousTask,
            NewClassTeacher, NewStudentClassAsynchronousTask,
        },
        schema::{
            class, class_asynchronous_task, class_student, class_synchronous_task, class_teacher,
            student_class_asynchronous_task,
        },
        utils::{client, login_user},
    };

    const CLASS_NAME: &str = "some-class";
    const CLASS_DESCRIPTION: &str = "some-class-description";

    const ASYNC_TASK_NAME: &str = "an asynchronous task";
    const ASYNC_TASK_DESCRIPTION: &str = "description of the asynchronous task";

    const SYNC_TASK_NAME: &str = "a synchronous task";
    const SYNC_TASK_DESCRIPTION: &str = "description of the synchronous task";

    async fn setup_env(conn: Database) {
        let (_, teacher_id, student_id, institution_id, student_group_id) =
            conn.run(|c| setup_env_to_be_extended(c)).await;
        conn.run(move |c| {
            let class_id = diesel::insert_into(class::table)
                .values(NewClass {
                    name: CLASS_NAME,
                    description: CLASS_DESCRIPTION,
                    created: Utc::now().naive_utc(),
                    code: &nanoid!(5),
                    institution_id: Some(institution_id),
                    student_group_id: Some(student_group_id),
                })
                .returning(class::id)
                .get_result::<i32>(c)
                .unwrap();
            let class_teacher_id = diesel::insert_into(class_teacher::table)
                .values(NewClassTeacher {
                    class_id,
                    user_id: teacher_id,
                })
                .returning(class_teacher::id)
                .get_result(c)
                .unwrap();
            let class_student_id = diesel::insert_into(class_student::table)
                .values(NewClassStudent {
                    user_id: student_id,
                    class_id,
                })
                .returning(class_student::id)
                .get_result(c)
                .unwrap();
            diesel::insert_into(class_synchronous_task::table)
                .values(NewClassSynchronousTask {
                    title: SYNC_TASK_NAME,
                    description: SYNC_TASK_DESCRIPTION,
                    created: Utc::now().naive_utc(),
                    start_time: Utc::now().add(Duration::days(5)).naive_utc(),
                    end_time: Utc::now()
                        .add(Duration::days(5))
                        .add(Duration::minutes(60))
                        .naive_utc(),
                    class_teacher_id,
                    class_id,
                })
                .execute(c)
                .unwrap();
            let task_id = diesel::insert_into(class_asynchronous_task::table)
                .values(NewClassAsynchronousTask {
                    title: ASYNC_TASK_NAME,
                    description: ASYNC_TASK_DESCRIPTION,
                    created: Utc::now().naive_utc(),
                    due_date: Utc::now().add(Duration::days(5)).naive_utc(),
                    class_teacher_id,
                    class_id,
                })
                .returning(class_asynchronous_task::id)
                .get_result(c)
                .unwrap();
            diesel::insert_into(student_class_asynchronous_task::table)
                .values(NewStudentClassAsynchronousTask {
                    class_student_id,
                    class_asynchronous_task_id: task_id,
                    completed: false,
                })
                .execute(c)
                .unwrap();
        })
        .await
    }

    #[rocket::async_test]
    async fn test_student_dashboard() {
        let client = client().await;
        setup_env(Database::get_one(client.rocket()).await.unwrap()).await;
        login_user(STUDENT_USERNAME, STUDENT_PASSWORD, &client).await;
        let dashboard_res = client.get("/dashboard").dispatch().await;
        let string = dashboard_res
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains(ASYNC_TASK_NAME));
        assert!(string.contains(ASYNC_TASK_DESCRIPTION));
        assert!(string.contains(SYNC_TASK_NAME));
        assert!(string.contains(SYNC_TASK_DESCRIPTION));
        assert!(!string.contains("0 of 1"));
    }
    #[rocket::async_test]
    async fn test_teacher_dashboard() {
        let client = client().await;
        setup_env(Database::get_one(client.rocket()).await.unwrap()).await;
        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client).await;
        let dashboard_res = client.get("/dashboard").dispatch().await;
        let string = dashboard_res
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains(ASYNC_TASK_NAME));
        assert!(string.contains(ASYNC_TASK_DESCRIPTION));
        assert!(string.contains(SYNC_TASK_NAME));
        assert!(string.contains(SYNC_TASK_DESCRIPTION));
        println!("{}", string);
        assert!(string.contains("0 of 1"));
    }
}
