/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

use diesel::prelude::*;
use malvolio::prelude::*;
use mercutio::Apply;
use portia::form::{FormStyle, FormSubmitInputStyle, FormTextInputStyle};
use rocket_contrib::json::Json;

use crate::{
    auth::AuthCookie,
    class::user_is_teacher,
    db::Database,
    models::{NewClassTeacherInvite, User},
    utils::{default_head, error::LovelaceError, error_message, json_response::ApiResponse},
};

fn invite_user_form() -> malvolio::prelude::Form {
    malvolio::prelude::Form::new()
        .apply(FormStyle)
        .attribute(Method::Post)
        .child(
            Input::default()
                .apply(FormTextInputStyle)
                .attribute(Type::Text)
                .attribute(Name::new("invited-user-identifier")),
        )
        .child(
            Input::default()
                .apply(FormSubmitInputStyle)
                .attribute(Type::Submit)
                .attribute(Value::new("Invite teacher!")),
        )
}

#[get("/class/<_id>/invite/teacher")]
pub fn invite_teacher_page(_id: usize) -> Html {
    Html::default()
        .head(default_head("Invite teacher".to_string()))
        .body(
            Body::default()
                .child(H1::new("Invite a new teacher"))
                .child(invite_user_form()),
        )
}

#[derive(FromForm, Debug, Clone)]
pub struct InviteTeacherForm {
    identifier: String,
}

#[post("/class/<id>/invite/teacher", data = "<form>")]
pub async fn html_invite_teacher(
    id: usize,
    auth_cookie: AuthCookie,
    form: rocket::request::Form<InviteTeacherForm>,
    conn: Database,
) -> Html {
    use crate::schema::class_teacher_invite::dsl as class_teacher_invite;
    use crate::schema::users::dsl as users;
    if !conn
        .run(move |c| user_is_teacher(auth_cookie.0, id as i32, c))
        .await
    {
        return Html::default().head(default_head("Permission denied".to_string())).body(
            Body::default()
                .child(H1::new("Invite a new teacher"))
                .child(P::with_text(
                    "You don't have permission to do that because you're not a teacher for this class ."
                ))
                .child(invite_user_form()),
        );
    }
    match conn
        .run(move |c| {
            users::users
                .filter(users::username.eq(&form.identifier))
                .or_filter(users::email.eq(&form.identifier))
                .first::<User>(c)
        })
        .await
    {
        Ok(user) => {
            match conn
                .run(move |c| {
                    diesel::insert_into(class_teacher_invite::class_teacher_invite)
                        .values(NewClassTeacherInvite {
                            inviting_user_id: auth_cookie.0,
                            invited_user_id: user.id,
                            class_id: id as i32,
                            accepted: false,
                        })
                        .execute(c)
                })
                .await
            {
                Ok(_) => Html::default()
                    .head(default_head("Header".to_string()))
                    .body(Body::default().child(H1::new("Successfully invited that user."))),
                Err(e) => {
                    error!("{:#?}", e);
                    error_message("Database error :(".to_string(),
                    "We've run into some problems with our database. This error has been logged and
                    we're working on fixing it.".to_string())
                }
            }
        }
        Err(diesel::result::Error::NotFound) => Html::default()
            .head(default_head("Invite a new teacher".to_string()))
            .body(
                Body::default()
                    .child(H1::new("Invite a new teacher"))
                    .child(P::with_text(
                        "A teacher with that username or email could not be found.",
                    ))
                    .child(invite_user_form()),
            ),
        Err(e) => {
            error!("{:?}", e);
            error_message(
                "Database error".to_string(),
                "Something's up with our database. We're working on fixing this.".to_string(),
            )
        }
    }
}

#[post("/class/<id>/invite/teacher", data = "<form>")]
pub async fn api_invite_teacher(
    id: usize,
    auth_cookie: AuthCookie,
    form: rocket::request::Form<InviteTeacherForm>,
    conn: Database,
) -> Json<ApiResponse<()>> {
    use crate::schema::class_teacher_invite::dsl as class_teacher_invite;
    use crate::schema::users::dsl as users;
    if !conn
        .run(move |c| user_is_teacher(auth_cookie.0, id as i32, c))
        .await
    {
        return Json(ApiResponse::new_err(
            "You don't have permission to invite other teachers because you're not a teacher for
            this class.",
        ));
    }
    match conn
        .run(move |c| {
            users::users
                .filter(users::username.eq(&form.identifier))
                .or_filter(users::email.eq(&form.identifier))
                .first::<User>(c)
        })
        .await
    {
        Ok(user) => {
            match conn
                .run(move |c| {
                    diesel::insert_into(class_teacher_invite::class_teacher_invite)
                        .values(NewClassTeacherInvite {
                            inviting_user_id: auth_cookie.0,
                            invited_user_id: user.id,
                            class_id: id as i32,
                            accepted: false,
                        })
                        .execute(c)
                })
                .await
            {
                Ok(_) => Json(ApiResponse::new_ok(())),
                Err(e) => {
                    error!("{:#?}", e);
                    Json(From::from(LovelaceError::DatabaseError))
                }
            }
        }
        Err(diesel::result::Error::NotFound) => Json(ApiResponse::new_err(
            "A teacher with those details could not be found.",
        )),
        Err(e) => {
            error!("{:?}", e);
            Json(From::from(LovelaceError::DatabaseError))
        }
    }
}

#[cfg(test)]
mod test_invite_teacher {
    use chrono::Utc;
    use diesel::prelude::*;
    use rocket::http::ContentType;

    use crate::{
        db::Database,
        models::{NewClass, NewClassTeacher, NewUser},
        schema::{class, class_teacher, class_teacher_invite, users},
        utils::{client, login_user},
    };

    pub const USERNAME: &str = "teacher";
    pub const EMAIL: &str = "teacher@example.com";
    pub const PASSWORD: &str = "s0mes3cuRE_passw-rd";
    pub const TIMEZONE: &str = "Africa/Abidjan";

    pub const USERNAME_2: &str = "other_teacher";
    pub const EMAIL_2: &str = "other_teacher@example.com";
    pub const PASSWORD_2: &str = "s3cuRE_passw-rd";

    const CLASS_NAME: &str = "name";
    const CLASS_DESCRIPTION: &str = "class-description";

    async fn setup_env(conn: Database) -> (i32, i32) {
        conn.run(|c| {
            let user_id = diesel::insert_into(users::table)
                .values(NewUser {
                    username: USERNAME,
                    email: EMAIL,
                    password: &bcrypt::hash(PASSWORD, bcrypt::DEFAULT_COST).unwrap(),
                    created: Utc::now().naive_utc(),
                    email_verified: true,
                    timezone: TIMEZONE,
                })
                .returning(users::id)
                .get_result(c)
                .unwrap();
            let class_id = diesel::insert_into(class::table)
                .values(NewClass {
                    name: CLASS_NAME,
                    description: CLASS_DESCRIPTION,
                    created: Utc::now().naive_utc(),
                    code: &nanoid!(5),
                    institution_id: None,
                    student_group_id: None,
                })
                .returning(class::id)
                .get_result(c)
                .unwrap();
            diesel::insert_into(class_teacher::table)
                .values(NewClassTeacher { user_id, class_id })
                .execute(c)
                .unwrap();
            diesel::insert_into(users::table)
                .values(NewUser {
                    username: USERNAME_2,
                    email: EMAIL_2,
                    password: &bcrypt::hash(PASSWORD_2, bcrypt::DEFAULT_COST).unwrap(),
                    created: Utc::now().naive_utc(),
                    email_verified: true,
                    timezone: TIMEZONE,
                })
                .execute(c)
                .unwrap();
            (user_id, class_id)
        })
        .await
    }

    #[rocket::async_test]
    async fn test_invite_teacher_html() {
        let client = client().await;
        let (_, class_id) = setup_env(Database::get_one(client.rocket()).await.unwrap()).await;
        login_user(USERNAME, PASSWORD, &client).await;
        let res = client
            .post(format!("/class/{}/invite/teacher", class_id))
            .header(ContentType::Form)
            .body(format!("identifier={}", USERNAME_2))
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains("invited that user"));
        assert!(Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(move |c| {
                diesel::select(diesel::dsl::exists(
                    class_teacher_invite::table
                        .inner_join(
                            users::table.on(users::id.eq(class_teacher_invite::invited_user_id)),
                        )
                        .filter(users::username.eq(USERNAME_2))
                        .filter(class_teacher_invite::class_id.eq(class_id)),
                ))
                .get_result::<bool>(c)
            })
            .await
            .unwrap_or(false));
    }

    #[rocket::async_test]

    async fn test_invite_teacher_api() {
        let client = client().await;
        let (_, class_id) = setup_env(Database::get_one(client.rocket()).await.unwrap()).await;
        login_user(USERNAME, PASSWORD, &client).await;
        let res = client
            .post(format!("/api/class/{}/invite/teacher", class_id))
            .header(ContentType::Form)
            .body(format!("identifier={}", USERNAME_2))
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains("\"success\":true"));
        assert!(Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(move |c| {
                diesel::select(diesel::dsl::exists(
                    class_teacher_invite::table
                        .inner_join(
                            users::table.on(users::id.eq(class_teacher_invite::invited_user_id)),
                        )
                        .filter(users::username.eq(USERNAME_2))
                        .filter(class_teacher_invite::class_id.eq(class_id)),
                ))
                .get_result::<bool>(c)
            })
            .await
            .unwrap_or(false));
    }
}
