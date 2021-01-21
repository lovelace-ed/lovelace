//! Allows teachers to send messages to students in the classes that they teach, as well as for
//! students to reply.
//!
//! This is a relatively simple model of chat for now – it is relatively email-like in that replies
//! are sequential. In future we should probably introduce both a threading model as well as the
//! option for classes to create small group chats to enable collaboration on assignments.

use diesel::prelude::*;
use diesel::BelongingToDsl;
use malvolio::{Body, Div, Form, Html, Input, H1, H3, P};
use rocket::{response::Redirect, FromForm};

use crate::{
    catch_database_error, css_names::LIST_ITEM, models::NewClassMessageReply,
    utils::html_or_redirect::HtmlOrRedirect,
};

use crate::{
    auth::AuthCookie,
    css_names::LIST,
    db::Database,
    models::{Class, ClassMessage, ClassMessageReply, NewClassMessage},
    utils::{default_head, error_messages::database_error},
};

use super::get_user_role_in_class;

#[get("/<id>/message")]
pub fn list_all_messages(id: i32, conn: Database, auth: AuthCookie) -> Html {
    use crate::schema::class::dsl as class;
    if let Some(_) = get_user_role_in_class(auth.0, id, &*conn) {
        let class =
            catch_database_error!(class::class.filter(class::id.eq(id)).first::<Class>(&*conn));
        let messages =
            catch_database_error!(ClassMessage::belonging_to(&class).load::<ClassMessage>(&*conn));
        Html::default()
            .head(default_head(format!("Messages in class {}", class.name)))
            .body(
                Body::default()
                    .child(H1::new(format!("Messages in class {}", class.name)))
                    .child(Div::default().attribute("class", LIST).children(
                        messages.into_iter().map(|message| {
                            Div::default()
                                .attribute("class", LIST_ITEM)
                                .child(H3::new(message.title))
                                .child(P::with_text(message.contents))
                        }),
                    )),
            )
    } else {
        Html::default()
            .head(default_head("Permission error".to_string()))
            .body(
                Body::default()
                    .child(H1::new("Permission error"))
                    .child(P::with_text(
                        "You don't have permission to view this class.",
                    )),
            )
    }
}

fn create_new_message_form() -> Form {
    Form::default()
        .attribute("method", "post")
        .child(
            Input::default()
                .attribute("type", "text")
                .attribute("name", "title"),
        )
        .child(
            Input::default()
                .attribute("type", "textarea")
                .attribute("name", "contents"),
        )
        .child(Input::default().attribute("type", "submit"))
}

#[get("/<id>/message/new")]
/// Returns the form which needs to be filled in and submitted in order to create a new user in a
/// class.
pub fn create_new_class_message(id: i32, auth: AuthCookie, conn: Database) -> Html {
    use crate::schema::class::dsl as class;
    use crate::schema::class_teacher::dsl as class_teacher;
    match class_teacher::class_teacher
        .filter(class_teacher::user_id.eq(auth.0))
        .filter(class_teacher::class_id.eq(id))
        .inner_join(class::class)
        .select(crate::schema::class::all_columns)
        .get_result::<Class>(&*conn)
    {
        Ok(class) => Html::default()
            .head(default_head(
                "Create a new message in this class".to_string(),
            ))
            .body(
                Body::default()
                    .child(H1::new(format!(
                        "Create a new message in class \"{class}\"",
                        class = class.name
                    )))
                    .child(create_new_message_form()),
            ),
        Err(diesel::result::Error::NotFound) => Html::default()
            .head(default_head("Error.".to_string()))
            .body(
                Body::default()
                    .child(H1::new("Error completing this request"))
                    .child(P::with_text(
                        "Either the class in question doesn't exist, or you aren't a
                teacher in that class.",
                    )),
            ),
        Err(e) => {
            error!("Error retrieving results from database: {:#?}", e);
            database_error()
        }
    }
}

#[derive(FromForm)]
pub struct CreateNewMessageForm {
    title: String,
    contents: String,
}

#[post("/<class_id>/message/new", data = "<form>")]
pub fn apply_create_new_class_message(
    class_id: i32,
    auth: AuthCookie,
    conn: Database,
    form: rocket::request::Form<CreateNewMessageForm>,
) -> HtmlOrRedirect {
    use crate::schema::class::dsl as class;
    use crate::schema::class_message::dsl as class_message;
    use crate::schema::class_teacher::dsl as class_teacher;
    match class_teacher::class_teacher
        .filter(class_teacher::user_id.eq(auth.0))
        .filter(class_teacher::class_id.eq(class_id))
        .inner_join(class::class)
        .select(crate::schema::class::all_columns)
        .get_result::<Class>(&*conn)
    {
        Ok(class) => class,
        Err(diesel::result::Error::NotFound) => {
            return HtmlOrRedirect::Html(
                Html::default()
                    .head(default_head("Error.".to_string()))
                    .body(
                        Body::default()
                            .child(H1::new("Error completing this request"))
                            .child(P::with_text(
                                "Either the class in question doesn't exist, or you aren't a
                teacher in that class.",
                            )),
                    ),
            )
        }
        Err(e) => {
            error!("{:#?}", e);
            return HtmlOrRedirect::Html(database_error());
        }
    };
    match diesel::insert_into(class_message::class_message)
        .values(NewClassMessage {
            title: &form.title,
            contents: &form.contents,
            created_at: chrono::Utc::now().naive_utc(),
            user_id: auth.0,
            class_id: class_id,
            edited: false,
        })
        .returning(class_message::id)
        .get_result::<i32>(&*conn)
    {
        Ok(returned_id) => HtmlOrRedirect::Redirect(Redirect::to(format!(
            "/class/{}/message/{}/view",
            class_id, returned_id
        ))),
        Err(e) => {
            error!("Error creating class page: {:#?}", e);
            HtmlOrRedirect::Html(database_error())
        }
    }
}

#[derive(FromForm)]
pub struct ReplyToTeacherMessageForm {
    contents: String,
}

#[post("/<class_id>/message/<message_id>/reply", data = "<form>")]
pub fn reply_to_teacher_message(
    class_id: i32,
    message_id: i32,
    auth: AuthCookie,
    conn: Database,
    form: rocket::request::Form<ReplyToTeacherMessageForm>,
) -> HtmlOrRedirect {
    use crate::schema::class_message_reply::dsl as class_message_reply;

    if get_user_role_in_class(auth.0, class_id, &*conn).is_none() {
        return HtmlOrRedirect::Html(Html::default());
    }

    match diesel::insert_into(class_message_reply::class_message_reply)
        .values(NewClassMessageReply {
            contents: &form.contents,
            created_at: chrono::Utc::now().naive_utc(),
            edited: false,
            user_id: auth.0,
            class_id,
            class_message_id: message_id,
        })
        .execute(&*conn)
    {
        Ok(_) => HtmlOrRedirect::Redirect(Redirect::to(format!(
            "/class/{}/message/{}/view",
            class_id, message_id
        ))),
        Err(e) => {
            error!("Error creating class message reply: {:#?}", e);
            HtmlOrRedirect::Html(database_error())
        }
    }
}

fn edit_message_form(msg: &ClassMessage) -> Form {
    Form::default()
        .child(
            Input::default()
                .attribute("type", "text")
                .attribute("name", "title")
                .attribute("value", msg.title.clone()),
        )
        .child(
            Input::default()
                .attribute("type", "textarea")
                .attribute("name", "contents")
                .attribute("value", msg.contents.clone()),
        )
        .child(Input::default().attribute("type", "submit"))
}

#[get("/<_class_id>/message/<message_id>/edit")]
pub fn edit_message(_class_id: i32, message_id: i32, conn: Database, auth: AuthCookie) -> Html {
    use crate::schema::class_message::dsl as class_message;
    match class_message::class_message
        .filter(class_message::id.eq(message_id))
        .filter(class_message::user_id.eq(auth.0))
        .first::<ClassMessage>(&*conn)
    {
        Ok(msg) => Html::default()
            .head(default_head("Insufficient permissions".to_string()))
            .body(
                Body::default()
                    .child(H1::new("Edit this message"))
                    .child(edit_message_form(&msg)),
            ),
        Err(e) => {
            error!("{:#?}", e);
            Html::default()
                .head(default_head("Insufficient permissions".to_string()))
                .body(
                    Body::default()
                        .child(H1::new("Insuffient permissions"))
                        .child(P::with_text(
                            "You didn't send this message, so you aren't allowed to edit it.",
                        )),
                )
        }
    }
}

#[derive(FromForm)]
pub struct EditMessageForm {
    title: String,
    contents: String,
}

#[post("/<class_id>/message/<message_id>/edit", data = "<form>")]
pub fn apply_message_edit(
    class_id: i32,
    message_id: i32,
    conn: Database,
    auth: AuthCookie,
    form: rocket::request::Form<EditMessageForm>,
) -> HtmlOrRedirect {
    use crate::schema::class_message::dsl as class_message;
    match diesel::update(
        class_message::class_message
            .filter(class_message::id.eq(message_id))
            .filter(class_message::user_id.eq(auth.0)),
    )
    .set((
        class_message::title.eq(&form.title),
        class_message::contents.eq(&form.contents),
    ))
    .execute(&*conn)
    {
        Ok(_) => HtmlOrRedirect::Redirect(Redirect::to(format!(
            "/class/{}/message/{}/view",
            class_id, message_id
        ))),
        Err(_) => HtmlOrRedirect::Html(database_error()),
    }
}

fn edit_message_reply_form(msg: &ClassMessageReply) -> Form {
    Form::default()
        .child(
            Input::default()
                .attribute("type", "text")
                .attribute("value", msg.contents.clone()),
        )
        .child(Input::default().attribute("type", "submit"))
}

#[get("/<class_id>/message/<_message_id>/reply/<message_reply_id>/edit")]
/// This _message_id is here as a placeholder (it is later used when returning a redirect after POST
/// request to the same location)
pub fn edit_message_reply(
    class_id: i32,
    message_reply_id: i32,
    _message_id: i32,
    conn: Database,
    auth: AuthCookie,
) -> Html {
    use crate::schema::class_message_reply::dsl as class_message_reply;
    match class_message_reply::class_message_reply
        .filter(class_message_reply::user_id.eq(auth.0))
        .filter(class_message_reply::class_id.eq(class_id))
        .filter(class_message_reply::id.eq(message_reply_id))
        .first::<ClassMessageReply>(&*conn)
    {
        Ok(msg) => Html::default()
            .head(default_head("Edit message".to_string()))
            .body(
                Body::default()
                    .child(H1::new("Edit your reply"))
                    .child(edit_message_reply_form(&msg)),
            ),
        Err(e) => {
            error!("Error rendering message reply edit form: {:#?}", e);
            database_error()
        }
    }
}

#[derive(FromForm)]
pub struct ApplyMessageReplyEditForm {
    contents: String,
}

#[post(
    "/<class_id>/message/<message_id>/reply/<message_reply_id>/edit",
    data = "<form>"
)]
pub fn apply_message_reply_edit(
    class_id: i32,
    message_reply_id: i32,
    message_id: i32,
    conn: Database,
    auth: AuthCookie,
    form: rocket::request::Form<ApplyMessageReplyEditForm>,
) -> HtmlOrRedirect {
    use crate::schema::class_message_reply::dsl as class_message_reply;
    match diesel::update(
        class_message_reply::class_message_reply
            .filter(class_message_reply::id.eq(message_reply_id))
            .filter(class_message_reply::class_id.eq(class_id))
            .filter(class_message_reply::user_id.eq(auth.0)),
    )
    .set(class_message_reply::contents.eq(&form.contents))
    .execute(&*conn)
    {
        Ok(_) => HtmlOrRedirect::Redirect(Redirect::to(format!(
            "/class/{}/message/{}/view",
            class_id, message_id
        ))),
        Err(e) => {
            error!("Error updating a reply to a message: {:#?}", e);
            HtmlOrRedirect::Html(database_error())
        }
    }
}

#[get("/<class_id>/message/<message_id>/view")]
pub fn view_message(class_id: i32, message_id: i32, auth: AuthCookie, conn: Database) -> Html {
    use crate::schema::class::dsl as class;
    use crate::schema::class_message::dsl as class_message;
    use crate::schema::users::dsl as users;
    let role = get_user_role_in_class(auth.0, class_id, &*conn);
    if role == None {
        return Html::default()
            .head(default_head(
                "You don't have permission to view this message.".to_string(),
            ))
            .body(
                Body::default()
                    .child(H1::new("You don't have permission to view this message"))
                    .child(P::with_text(
                        "You might need to ask your teacher for an invite to this class.",
                    )),
            );
    }
    match class_message::class_message
        .filter(class_message::id.eq(message_id))
        .inner_join(class::class)
        .filter(class::id.eq(class_id))
        .select(crate::schema::class_message::all_columns)
        .first::<ClassMessage>(&*conn)
    {
        Ok(msg) => {
            match ClassMessageReply::belonging_to(&msg)
                .inner_join(crate::schema::users::table)
                .select((
                    crate::schema::class_message_reply::all_columns,
                    users::username,
                ))
                .load::<(ClassMessageReply, String)>(&*conn)
            {
                Ok(replies) => Html::default().head(default_head("".to_string())).body(
                    Body::default().children(replies.into_iter().map(|(reply, username)| {
                        Div::default()
                            .attribute("class", LIST)
                            .child(H3::new(format!("Reply from {}", username)))
                            .child(P::with_text(format!(
                                "This reply was posted at {}",
                                reply.created_at.to_string()
                            )))
                            .child(P::with_text(reply.contents))
                    })),
                ),
                Err(e) => {
                    error!("Database error loading replies: {:#?}", e);
                    database_error()
                }
            }
        }
        Err(e) => {
            error!("Error retrieving class message {:#?}", e);
            database_error()
        }
    }
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn test_can_create_class_message() {
        const MESSAGE_TITLE: &str = "sometitleofatitle";
        const MESSAGE_BODY: &str = "somebodyof a message";
        let client = client();
        let (class_id, _, _, _) = setup_test_env(&*Database::get_one(&client.rocket()).unwrap());
        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client);
        let create_message_res = client
            .post(format!("/class/{}/message/new", class_id))
            .header(ContentType::Form)
            .body(format!("title={}&contents={}", MESSAGE_TITLE, MESSAGE_BODY))
            .dispatch();
        assert_eq!(create_message_res.status().code, 303);
        {
            use crate::schema::class_message::dsl as class_message;
            let res = class_message::class_message
                .filter(class_message::title.eq(MESSAGE_TITLE))
                .filter(class_message::contents.eq(MESSAGE_BODY))
                .first::<ClassMessage>(&*Database::get_one(&client.rocket()).unwrap())
                .expect("could not find");
            assert_eq!(res.title, MESSAGE_TITLE);
            assert_eq!(res.contents, MESSAGE_BODY);
        }
    }
    #[test]
    fn test_can_edit_class_message() {
        const NEW_TITLE: &str = "new-title";
        const NEW_CONTENTS: &str = "new-contents-here-we-come";

        let client = client();
        let (class_id, message_ids, _, _) =
            setup_test_env(&*Database::get_one(&client.rocket()).unwrap());

        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client);

        let edit_class_message_res = client
            .post(format!(
                "/class/{}/message/{}/edit",
                class_id, message_ids[0]
            ))
            .header(ContentType::Form)
            .body(format!("title={}&contents={}", NEW_TITLE, NEW_CONTENTS))
            .dispatch();
        assert!(edit_class_message_res.status().code == 303);

        {
            use crate::schema::class_message::dsl as class_message;
            let msg = class_message::class_message
                .filter(class_message::id.eq(message_ids[0]))
                .first::<ClassMessage>(&*Database::get_one(client.rocket()).unwrap())
                .expect("error loading results");
            assert_eq!(msg.title, NEW_TITLE);
            assert_eq!(msg.contents, NEW_CONTENTS);
        }
    }
    #[test]
    fn test_can_view_messages() {
        let client = client();
        let (class_id, _, _, _) = setup_test_env(&*Database::get_one(&client.rocket()).unwrap());
        login_user(STUDENT_EMAIL, STUDENT_PASSWORD, &client);
        let mut view_message_res = client
            .get(format!("/class/{}/message", class_id))
            .dispatch();
        let string = view_message_res
            .body_string()
            .expect("invalid body response");

        assert!(string.contains(CLASS_MESSAGE_1_TITLE));
        assert!(string.contains(CLASS_MESSAGE_1_CONTENTS));

        assert!(string.contains(CLASS_MESSAGE_2_TITLE));
        assert!(string.contains(CLASS_MESSAGE_2_CONTENTS));
    }
    #[test]
    fn test_reply_to_class_message() {
        const REPLY_CONTENTS: &str = "somereplycontents235";
        let client = client();
        let (class_id, message_ids, _, _) =
            setup_test_env(&*Database::get_one(&client.rocket()).unwrap());
        login_user(STUDENT_EMAIL, STUDENT_PASSWORD, &client);
        let reply_res = client
            .post(format!(
                "/class/{}/message/{}/reply",
                class_id, message_ids[0]
            ))
            .header(ContentType::Form)
            .body(format!("contents={}", REPLY_CONTENTS))
            .dispatch();
        assert_eq!(reply_res.status().code, 303);
        let mut message_page = client
            .get(format!(
                "/class/{}/message/{}/view",
                class_id, message_ids[0]
            ))
            .dispatch();
        let string = message_page.body_string().expect("invalid body response");
        assert!(string.contains(REPLY_CONTENTS));
    }
    #[test]
    fn test_can_edit_reply_to_class_message() {
        const NEW_MESSAGE_CONTENTS: &str = "somecontents that is new";
        let client = client();
        let (class_id, message_ids, student_id, _) =
            setup_test_env(&*Database::get_one(&client.rocket()).unwrap());
        login_user(STUDENT_USERNAME, STUDENT_PASSWORD, &client);
        let message_reply_id = add_message_reply(
            message_ids[0],
            student_id,
            class_id,
            &*Database::get_one(&client.rocket()).unwrap(),
        );
        let edit_message_res = client
            .post(format!(
                "/class/{}/message/{}/reply/{}/edit",
                class_id, message_ids[0], message_reply_id
            ))
            .header(ContentType::Form)
            .body(format!("contents={}", NEW_MESSAGE_CONTENTS))
            .dispatch();
        assert_eq!(edit_message_res.status().code, 303);
        let mut view_message_replies = client
            .get(format!(
                "/class/{}/message/{}/view",
                class_id, message_ids[0]
            ))
            .dispatch();
        let string = view_message_replies
            .body_string()
            .expect("invalid body response");
        println!("{}", string);
        assert!(string.contains(NEW_MESSAGE_CONTENTS));
    }
}
