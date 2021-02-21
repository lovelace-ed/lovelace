/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

use diesel::prelude::*;

pub mod configure;
pub mod create;
pub mod delete;
pub mod invite;
pub mod join;
pub mod list;
pub mod members;
pub mod messages;
pub mod overview;
pub mod tasks;

pub use configure::get_class_settings;
pub use create::{api_create_class, create_class_page, html_create_class};
pub use delete::{api_delete_class, delete_class_page, html_delete_class};
pub use invite::{api_invite_teacher, html_invite_teacher, invite_teacher_page};
pub use join::{api_join_class, html_join_class};
pub use list::{api_view_all_classes, html_view_all_classes};
pub use members::{api_view_class_members_page, html_view_class_members_page};
pub use overview::api_view_class_overview;

use crate::db::{Database, DatabaseConnection};

#[derive(Debug, Eq, PartialEq)]
pub enum ClassMemberRole {
    Teacher,
    Student,
}

/// Returns the role that a given user has in a given class.
pub async fn get_user_role_in_class(
    user: i32,
    class: i32,
    conn: &Database,
) -> Option<ClassMemberRole> {
    use crate::schema::class_student::dsl as class_student;
    use crate::schema::class_teacher::dsl as class_teacher;
    let if_condition = conn
        .run(move |c| {
            diesel::dsl::select(diesel::dsl::exists(
                class_student::class_student
                    .filter(class_student::user_id.eq(user))
                    .filter(class_student::class_id.eq(class)),
            ))
            .get_result(c)
        })
        .await
        .unwrap();
    let else_if_condition = conn.run(move |c| {
        diesel::dsl::select(diesel::dsl::exists(
            class_teacher::class_teacher
                .filter(class_teacher::user_id.eq(user))
                .filter(class_teacher::class_id.eq(class)),
        ))
        .get_result(c)
    });
    if if_condition {
        Some(ClassMemberRole::Student)
    } else if else_if_condition.await.unwrap() {
        Some(ClassMemberRole::Teacher)
    } else {
        None
    }
}

pub fn user_is_teacher(user_id: i32, class_id: i32, conn: &DatabaseConnection) -> bool {
    use crate::schema::class_teacher::dsl as class_teacher;
    diesel::dsl::select(diesel::dsl::exists(
        class_teacher::class_teacher
            .filter(class_teacher::user_id.eq(user_id))
            .filter(class_teacher::class_id.eq(class_id)),
    ))
    .get_result(&*conn)
    .map_err(|e| error!("{:#?}", e))
    .unwrap_or(false)
}

#[cfg(test)]
mod test_class_routes {
    use regex::Regex;
    use rocket::http::ContentType;

    use crate::utils::{create_user, login_user, logout};

    const TIMEZONE: &str = "Africa/Abidjan";
    const TEACHER_USERNAME: &str = "some_teacher";
    const TEACHER_EMAIL: &str = "some_teacher@example.com";
    const TEACHER_PASSWORD: &str = "somePASSW0RD123";
    const SECONDARY_TEACHER_USERNAME: &str = "some_secondary_teacher";
    const SECONDARY_TEACHER_EMAIL: &str = "some_secondary_teacher@example.com";
    const SECONDARY_TEACHER_PASSWORD: &str = "SomeSEcondARyT3@CHER";
    const STUDENT_USERNAME: &str = "student_aw";
    const STUDENT_EMAIL: &str = "student@example.com";
    const STUDENT_PASSWORD: &str = "stUD3NTP@SSW0RD";
    const CLASS_NAME: &str = "Some class name";
    const CLASS_DESCRIPTION: &str = "Some class description";

    #[rocket::async_test]
    async fn test_class_handling() {
        let client = crate::utils::client().await;
        create_user(
            TEACHER_USERNAME,
            TEACHER_EMAIL,
            TIMEZONE,
            TEACHER_PASSWORD,
            &client,
        )
        .await;
        create_user(
            SECONDARY_TEACHER_USERNAME,
            SECONDARY_TEACHER_EMAIL,
            TIMEZONE,
            SECONDARY_TEACHER_PASSWORD,
            &client,
        )
        .await;
        create_user(
            STUDENT_USERNAME,
            STUDENT_EMAIL,
            TIMEZONE,
            STUDENT_PASSWORD,
            &client,
        )
        .await;

        // test can create class
        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client).await;
        let create_class_res = client.get("/class/create").dispatch().await;
        let string = create_class_res
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains("Create a class"));

        let create_class_res = client
            .post("/class/create")
            .header(ContentType::Form)
            .body(format!(
                "name={}&description={}",
                CLASS_NAME, CLASS_DESCRIPTION
            ))
            .dispatch()
            .await;
        assert!(create_class_res
            .into_string()
            .await
            .expect("invalid body response")
            .contains("Successfully created"));

        // test created class shows up on teacher class list
        let get_class_list = client.get("/class").dispatch().await;
        let string = get_class_list
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains(CLASS_NAME));

        let id = Regex::new(r#"class/(?P<id>[0-9]+)"#)
            .unwrap()
            .captures(&string)
            .unwrap()
            .name("id")
            .unwrap()
            .as_str()
            .parse::<i32>()
            .unwrap();

        // test created class overview page can be seen

        let class_overview_page = client.get(format!("/class/{}", id)).dispatch().await;
        let string = class_overview_page
            .into_string()
            .await
            .expect("invalid body string");
        assert!(string.contains(CLASS_NAME));
        assert!(string.contains(CLASS_DESCRIPTION));
        let join_code =
            Regex::new(r#"Invite people to join with the code: (?P<code>[a-zA-Z0-9~_]+)"#)
                .unwrap()
                .captures(&string)
                .unwrap()
                .name("code")
                .unwrap()
                .as_str();

        // test teacher can see settings page

        let settings_page = client
            .get(format!("/class/{}/settings", id))
            .dispatch()
            .await;
        let string = settings_page.into_string().await.unwrap();
        assert!(string.contains("delete"));

        // test students cannot join classes with the incorrect code
        logout(&client).await;

        login_user(STUDENT_EMAIL, STUDENT_PASSWORD, &client).await;

        let invalid_join_attempt = client
            .get("/join/SOME_RANDOM_CODE_WHICH_DOES_NOT_WORK+")
            .dispatch()
            .await;
        let string = invalid_join_attempt.into_string().await.unwrap();
        assert!(string.contains("cannot be found"));

        // test students can join class

        let valid_join_attempt = client.get(format!("/join/{}", join_code)).dispatch().await;
        let string = valid_join_attempt.into_string().await.unwrap();
        assert!(string.contains("joined this class"));

        // test joined classes show up on student class list

        let student_class_list = client.get("/class".to_string()).dispatch().await;
        let string = student_class_list
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains(CLASS_NAME));

        // test students can see class overview page

        let class_overview_page = client.get(format!("/class/{}", id)).dispatch().await;
        let string = class_overview_page
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains(CLASS_NAME));
        assert!(!string.contains("people to join"));

        // test teacher can delete class from the settings page

        logout(&client).await;

        login_user(TEACHER_EMAIL, TEACHER_PASSWORD, &client).await;

        let delete_page = client.get(format!("/class/{}/delete", id)).dispatch().await;
        let string = delete_page
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains("Delete this class"));

        // test can't delete class without correct name

        let invalid_delete_request = client
            .post("/class/delete".to_string())
            .header(ContentType::Form)
            .body(format!("id={}&confirm_name=wrong", id))
            .dispatch()
            .await;
        let string = invalid_delete_request
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains("doesn't match"));

        // test can't delete class without correct class id

        let invalid_delete_request = client
            .post("/class/delete".to_string())
            .header(ContentType::Form)
            .body(format!("id={}&confirm_name={}", 100000000, CLASS_NAME))
            .dispatch()
            .await;
        let string = invalid_delete_request
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains("Permission denied"));

        // test teacher can delete class

        let invalid_delete_request = client
            .post("/class/delete".to_string())
            .header(ContentType::Form)
            .body(format!("id={}&confirm_name={}", id, CLASS_NAME))
            .dispatch()
            .await;
        let string = invalid_delete_request
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains("sucessfully deleted"));

        // test teacher can't see deleted classes

        let class_overview_page = client.get(format!("/client/{}", id)).dispatch().await;
        let string = class_overview_page
            .into_string()
            .await
            .expect("invalid body string");
        assert!(!string.contains(CLASS_NAME));
        assert!(!string.contains(CLASS_DESCRIPTION));

        // test students can't see deleted classes

        logout(&client).await;
        login_user(STUDENT_EMAIL, STUDENT_PASSWORD, &client).await;
        let class_overview_page = client.get(format!("/client/{}", id)).dispatch().await;
        let string = class_overview_page
            .into_string()
            .await
            .expect("invalid body string");
        assert!(!string.contains(CLASS_NAME));
        assert!(!string.contains(CLASS_DESCRIPTION));
    }
}
