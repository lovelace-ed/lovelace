/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

//! Allows teachers to send messages to students in the classes that they teach, as well as for
//! students to reply.
//!
//! This is a relatively simple model of chat for now – it is relatively email-like in that replies
//! are sequential. In future we should probably introduce both a threading model as well as the
//! option for classes to create small group chats to enable collaboration on assignments.
//!
//! Support is also planned for server-sent events to subscribe to updates, but I think we're
//! waiting on Rocket adding support for them first.

mod create;
mod edit;
mod list;
mod view;

pub use create::{
    message::{
        api_apply_create_new_class_message, create_new_class_message_page,
        html_apply_create_new_class_message,
    },
    reply::{api_reply_to_teacher_message, html_reply_to_teacher_message},
};
pub use edit::{
    message::{api_apply_message_edit, edit_message_page, html_apply_message_edit},
    reply::{api_apply_message_reply_edit, edit_message_reply, html_apply_message_reply_edit},
};
pub use list::{api_list_all_messages, html_list_all_messages};
pub use view::{api_view_message, view_message};

#[cfg(test)]
mod class_tests {
    use rocket::http::ContentType;

    use diesel::prelude::*;

    use crate::{
        db::{Database, DatabaseConnection},
        models::{
            Class, ClassMessage, ClassMessageReply, NewClass, NewClassMessage,
            NewClassMessageReply, NewClassStudent, NewClassTeacher, NewUser, User,
        },
        utils::{client, login_user},
    };

    const TIMEZONE: &str = "Africa/Abidjan";

    const CLASS_NAME: &str = "classname";
    const CLASS_DESCRIPTION: &str = "class description";

    const TEACHER_USERNAME: &str = "someteacherusername";
    const TEACHER_EMAIL: &str = "someteacher@example.com";
    const TEACHER_PASSWORD: &str = "Passw0rd123";

    const STUDENT_USERNAME: &str = "some-student";
    const STUDENT_EMAIL: &str = "some@students.example.com";
    const STUDENT_PASSWORD: &str = "VeryL0ngAndV3ryS3cur3";

    const CLASS_MESSAGE_1_TITLE: &str = "first-class-messages";
    const CLASS_MESSAGE_1_CONTENTS: &str = "somesortof0fonc";

    const CLASS_MESSAGE_2_TITLE: &str = "first-class-messages";
    const CLASS_MESSAGE_2_CONTENTS: &str = "thridy1243";

    const CLASS_MESSAGE_REPLY_ORIGINAL_CONTENTS: &str = "somenessssss34";

    /// Returns a tuple of (class_id, vec<message ids>, student id, teacher id)
    fn setup_test_env(conn: &DatabaseConnection) -> (i32, Vec<i32>, i32, i32) {
        use crate::schema::class::dsl as class;
        use crate::schema::class_message::dsl as class_message;
        use crate::schema::class_student::dsl as class_student;
        use crate::schema::class_teacher::dsl as class_teacher;
        use crate::schema::users::dsl as users;

        let teacher = diesel::insert_into(users::users)
            .values(NewUser {
                username: TEACHER_USERNAME,
                email: TEACHER_EMAIL,
                password: &bcrypt::hash(TEACHER_PASSWORD, bcrypt::DEFAULT_COST).unwrap(),
                created: chrono::Utc::now().naive_utc(),
                email_verified: true,
                timezone: TIMEZONE,
            })
            .returning(crate::schema::users::all_columns)
            .get_result::<User>(conn)
            .unwrap();
        let student = diesel::insert_into(users::users)
            .values(NewUser {
                username: STUDENT_USERNAME,
                email: STUDENT_EMAIL,
                password: &bcrypt::hash(STUDENT_PASSWORD, bcrypt::DEFAULT_COST).unwrap(),
                created: chrono::Utc::now().naive_utc(),
                email_verified: true,
                timezone: TIMEZONE,
            })
            .returning(crate::schema::users::all_columns)
            .get_result::<User>(conn)
            .unwrap();
        let class = diesel::insert_into(class::class)
            .values(NewClass {
                name: CLASS_NAME,
                description: CLASS_DESCRIPTION,
                created: chrono::Utc::now().naive_utc(),
                code: &nanoid!(5),
                institution_id: None,
                student_group_id: None,
            })
            .returning(crate::schema::class::all_columns)
            .get_result::<Class>(conn)
            .unwrap();
        diesel::insert_into(class_student::class_student)
            .values(NewClassStudent {
                user_id: student.id,
                class_id: class.id,
            })
            .execute(conn)
            .unwrap();
        diesel::insert_into(class_teacher::class_teacher)
            .values(NewClassTeacher {
                user_id: teacher.id,
                class_id: class.id,
            })
            .execute(conn)
            .unwrap();
        let message_1 = diesel::insert_into(class_message::class_message)
            .values(NewClassMessage {
                title: CLASS_MESSAGE_1_TITLE,
                contents: CLASS_MESSAGE_1_CONTENTS,
                created_at: chrono::Utc::now().naive_utc(),
                user_id: teacher.id,
                class_id: class.id,
                edited: false,
            })
            .returning(crate::schema::class_message::all_columns)
            .get_result::<ClassMessage>(conn)
            .unwrap();
        let message_2 = diesel::insert_into(class_message::class_message)
            .values(NewClassMessage {
                title: CLASS_MESSAGE_2_TITLE,
                contents: CLASS_MESSAGE_2_CONTENTS,
                created_at: chrono::Utc::now().naive_utc(),
                user_id: teacher.id,
                class_id: class.id,
                edited: false,
            })
            .returning(crate::schema::class_message::all_columns)
            .get_result::<ClassMessage>(conn)
            .unwrap();
        (
            class.id,
            vec![message_1.id, message_2.id],
            student.id,
            teacher.id,
        )
    }
    fn add_message_reply(
        message_id: i32,
        user_id: i32,
        class_id: i32,
        conn: &DatabaseConnection,
    ) -> i32 {
        use crate::schema::class_message_reply::dsl as class_message_reply;

        let msg_reply = diesel::insert_into(class_message_reply::class_message_reply)
            .values(NewClassMessageReply {
                contents: CLASS_MESSAGE_REPLY_ORIGINAL_CONTENTS,
                created_at: chrono::Utc::now().naive_utc(),
                edited: false,
                user_id,
                class_id,
                class_message_id: message_id,
            })
            .returning(crate::schema::class_message_reply::all_columns)
            .get_result::<ClassMessageReply>(conn)
            .unwrap();
        msg_reply.id
    }

    #[rocket::async_test]
    async fn test_can_create_class_message() {
        const MESSAGE_TITLE: &str = "sometitleofatitle";
        const MESSAGE_BODY: &str = "somebodyof a message";
        let client = client().await;
        let (class_id, _, _, _) = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| setup_test_env(c))
            .await;
        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client).await;
        let create_message_res = client
            .post(format!("/class/{}/message/new", class_id))
            .header(ContentType::Form)
            .body(format!("title={}&contents={}", MESSAGE_TITLE, MESSAGE_BODY))
            .dispatch()
            .await;
        assert_eq!(create_message_res.status().code, 303);
        {
            use crate::schema::class_message::dsl as class_message;
            let res = Database::get_one(&client.rocket())
                .await
                .unwrap()
                .run(|c| {
                    class_message::class_message
                        .filter(class_message::title.eq(MESSAGE_TITLE))
                        .filter(class_message::contents.eq(MESSAGE_BODY))
                        .first::<ClassMessage>(c)
                        .expect("could not find")
                })
                .await;
            assert_eq!(res.title, MESSAGE_TITLE);
            assert_eq!(res.contents, MESSAGE_BODY);
        }
    }
    #[rocket::async_test]
    async fn test_can_edit_class_message() {
        const NEW_TITLE: &str = "new-title";
        const NEW_CONTENTS: &str = "new-contents-here-we-come";

        let client = client().await;
        let (class_id, message_ids, _, _) = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| setup_test_env(c))
            .await;
        let message_id_0 = message_ids[0];
        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client).await;

        let edit_class_message_res = client
            .post(format!("/class/{}/message/{}/edit", class_id, message_id_0))
            .header(ContentType::Form)
            .body(format!("title={}&contents={}", NEW_TITLE, NEW_CONTENTS))
            .dispatch()
            .await;
        assert!(edit_class_message_res.status().code == 303);

        {
            use crate::schema::class_message::dsl as class_message;
            let msg = Database::get_one(client.rocket())
                .await
                .unwrap()
                .run(move |c| {
                    class_message::class_message
                        .filter(class_message::id.eq(message_id_0))
                        .first::<ClassMessage>(c)
                })
                .await
                .expect("error loading results");
            assert_eq!(msg.title, NEW_TITLE);
            assert_eq!(msg.contents, NEW_CONTENTS);
        }
    }
    #[rocket::async_test]
    async fn test_can_view_messages() {
        let client = client().await;
        let (class_id, _, _, _) = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| setup_test_env(c))
            .await;
        login_user(STUDENT_EMAIL, STUDENT_PASSWORD, &client).await;
        let view_message_res = client
            .get(format!("/class/{}/message", class_id))
            .dispatch()
            .await;
        let string = view_message_res
            .into_string()
            .await
            .expect("invalid body response");

        assert!(string.contains(CLASS_MESSAGE_1_TITLE));
        assert!(string.contains(CLASS_MESSAGE_1_CONTENTS));

        assert!(string.contains(CLASS_MESSAGE_2_TITLE));
        assert!(string.contains(CLASS_MESSAGE_2_CONTENTS));
    }
    #[rocket::async_test]
    async fn test_reply_to_class_message() {
        const REPLY_CONTENTS: &str = "somereplycontents235";
        let client = client().await;
        let (class_id, message_ids, _, _) = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| setup_test_env(c))
            .await;
        login_user(STUDENT_EMAIL, STUDENT_PASSWORD, &client).await;
        let reply_res = client
            .post(format!(
                "/class/{}/message/{}/reply",
                class_id, message_ids[0]
            ))
            .header(ContentType::Form)
            .body(format!("contents={}", REPLY_CONTENTS))
            .dispatch()
            .await;
        assert_eq!(reply_res.status().code, 303);
        let message_page = client
            .get(format!(
                "/class/{}/message/{}/view",
                class_id, message_ids[0]
            ))
            .dispatch()
            .await;
        let string = message_page
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains(REPLY_CONTENTS));
    }
    #[rocket::async_test]
    async fn test_can_edit_reply_to_class_message() {
        const NEW_MESSAGE_CONTENTS: &str = "somecontents that is new";
        let client = client().await;
        let (class_id, message_ids, student_id, _) = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(|c| setup_test_env(c))
            .await;
        login_user(STUDENT_USERNAME, STUDENT_PASSWORD, &client).await;
        let message_id_1 = message_ids[0];
        let message_reply_id = Database::get_one(&client.rocket())
            .await
            .unwrap()
            .run(move |c| add_message_reply(message_id_1, student_id, class_id, c))
            .await;
        let edit_message_res = client
            .post(format!(
                "/class/{}/message/{}/reply/{}/edit",
                class_id, message_id_1, message_reply_id
            ))
            .header(ContentType::Form)
            .body(format!("contents={}", NEW_MESSAGE_CONTENTS))
            .dispatch()
            .await;
        assert_eq!(edit_message_res.status().code, 303);
        let view_message_replies = client
            .get(format!("/class/{}/message/{}/view", class_id, message_id_1))
            .dispatch()
            .await;
        let string = view_message_replies
            .into_string()
            .await
            .expect("invalid body response");
        assert!(string.contains(NEW_MESSAGE_CONTENTS));
    }
}
