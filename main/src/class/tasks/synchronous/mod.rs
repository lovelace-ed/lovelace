/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
//! Synchronous tasks (e.g. homework).

use crate::auth::AuthCookie;

pub mod create;
pub mod delete;
pub mod edit;
pub mod list;
pub mod view;

pub use create::{api_create_new_async_task, get_create_new_sync_task, html_create_new_sync_task};
pub use delete::{api_delete_task, html_delete_task};
pub use edit::{api_apply_edit_task, html_apply_edit_task, view_edit_task_page};
pub use list::{api_view_all_sync_tasks_in_class, html_view_all_sync_tasks_in_class};
pub use view::{api_view_specific_synchronous_task, html_view_specific_synchronous_task};

#[cfg(test)]
mod synchronous_task_tests {
    use std::ops::Add;

    use crate::{
        db::{Database, DatabaseConnection},
        models::{
            ClassSynchronousTask, NewClassStudent, NewClassSynchronousTask, NewClassTeacher,
            NewStudentClassSynchronousTask, StudentClassSynchronousTask,
        },
        utils::{client, login_user, logout},
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
                institution_id: None,
                student_group_id: None,
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
    #[rocket::async_test]
    async fn test_teacher_can_view_specific_synchronous_task() {
        let client = client().await;
        let (class_id, _, _, tasks) = Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;
        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client).await;
        let view_task_res = client
            .get(format!("/class/{}/task/sync/{}/view", class_id, tasks[0]))
            .dispatch()
            .await;
        let string = view_task_res
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains(TASK_1_TITLE));
        assert!(string.contains(TASK_1_DESCRIPTION));
    }
    #[rocket::async_test]
    async fn test_student_can_view_specific_synchronous_task() {
        let client = client().await;
        let (class_id, _, _, tasks) = Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;

        login_user(STUDENT_1_USERNAME, STUDENT_1_PASSWORD, &client).await;
        let view_task_res = client
            .get(format!("/class/{}/task/sync/{}/view", class_id, tasks[0]))
            .dispatch()
            .await;
        let string = view_task_res
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains(TASK_1_TITLE));
        assert!(string.contains(TASK_1_DESCRIPTION));

        login_user(STUDENT_2_USERNAME, STUDENT_2_PASSWORD, &client).await;
        let view_task_res = client
            .get(format!("/class/{}/task/sync/{}/view", class_id, tasks[0]))
            .dispatch()
            .await;
        let string = view_task_res
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains(TASK_1_TITLE));
        assert!(string.contains(TASK_1_DESCRIPTION));
    }
    #[rocket::async_test]
    async fn test_teacher_can_create_synchronous_task() {
        const NEW_TASK_TITLE: &str = "new-task-title";
        const NEW_TASK_DESCRIPTION: &str = "new-task-description";
        let client = client().await;
        let (class_id, _, _, _) = Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;
        login_user(TEACHER_EMAIL, TEACHER_PASSWORD, &client).await;

        let res = client
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
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains("Created that task"));
        {
            use crate::schema::class_synchronous_task::dsl as class_synchronous_task;
            use crate::schema::student_class_synchronous_task::dsl as student_class_synchronous_task;

            let results = Database::get_one(client.rocket())
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
    #[rocket::async_test]
    async fn test_teacher_can_edit_synchronous_task() {
        const NEW_TASK_TITLE: &str = "new-task-title";
        const NEW_TASK_DESCRIPTION: &str = "new-task-description";
        let client = client().await;
        let (class_id, _, _, tasks) = Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;
        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client).await;
        let res = client
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
        let (class_id, _, _, tasks) = Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;
        login_user(STUDENT_1_USERNAME, STUDENT_1_PASSWORD, &client).await;
        let res = client
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
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(!string.contains("updated that task"));
    }
    #[rocket::async_test]
    async fn test_teacher_can_delete_synchronous_task() {
        let client = client().await;
        let (class_id, _, _, tasks) = Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;
        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client).await;
        let res = client
            .get(format!("/class/{}/task/sync/{}/delete", class_id, tasks[1]))
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains("deleted that task"));
    }
    #[rocket::async_test]
    async fn test_permissions_for_viewing_create_task_page() {
        let client = client().await;
        let (class_id, _, _, _) = Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;
        login_user(STUDENT_1_USERNAME, STUDENT_1_PASSWORD, &client).await;
        let res = client
            .get(format!("/class/{}/task/sync/create", class_id))
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains("don't have permission"));
        logout(&client).await;
        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client).await;
        let res = client
            .get(format!("/class/{}/task/sync/create", class_id))
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains("<form"));
    }
    #[rocket::async_test]
    async fn test_permissions_for_viewing_edit_task_page() {
        let client = client().await;
        let (class_id, _, _, tasks) = Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;

        login_user(STUDENT_1_USERNAME, STUDENT_1_PASSWORD, &client).await;
        let res = client
            .get(format!("/class/{}/task/sync/{}/edit", class_id, tasks[0]))
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains("don't have permission"));
        logout(&client).await;

        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client).await;
        let res = client
            .get(format!("/class/{}/task/sync/{}/edit", class_id, tasks[0]))
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains(TASK_1_TITLE));
        assert!(string.contains(TASK_1_DESCRIPTION))
    }
    #[rocket::async_test]
    async fn test_view_task_summary_page() {
        let client = client().await;
        let (class_id, _, _, _) = Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(|c| populate_database(c))
            .await;
        login_user(STUDENT_1_USERNAME, STUDENT_1_PASSWORD, &client).await;
        let res = client
            .get(format!("/class/{}/task/sync/all", class_id))
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains(TASK_1_TITLE));
        assert!(string.contains(TASK_1_DESCRIPTION));
        assert!(string.contains(TASK_2_DESCRIPTION));
        assert!(string.contains(TASK_2_TITLE));
        logout(&client).await;
        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client).await;
        let res = client
            .get(format!("/class/{}/task/sync/all", class_id))
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains(TASK_1_TITLE));
        assert!(string.contains(TASK_1_DESCRIPTION));
        assert!(string.contains(TASK_2_DESCRIPTION));
        assert!(string.contains(TASK_2_TITLE));
        assert!(string.contains(&format!("Set by: {}", TEACHER_USERNAME)));
    }
}
