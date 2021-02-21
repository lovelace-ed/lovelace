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
