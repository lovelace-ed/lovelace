use crate::{
    auth::AuthCookie,
    db::Database,
    models::{ClassMessageReply, NewClassMessageReply},
    utils::html_or_redirect::HtmlOrRedirect,
};
use crate::{class::get_user_role_in_class, utils::json_response::ApiResponse};

use crate::utils::error_messages::database_error;
use crate::utils::permission_error::permission_error;
use diesel::prelude::*;
use rocket::response::Redirect;
use rocket_contrib::json::Json;
use thiserror::Error as ThisError;

#[derive(FromForm, Debug, Clone, Serialize, Deserialize)]
pub struct ReplyToTeacherMessageForm {
    contents: String,
}

#[derive(ThisError, Debug)]
pub enum AddReplyError {
    #[error("permission error")]
    PermissionError,
    #[error("database error")]
    DatabaseError,
}

async fn add_reply_to_teacher_message_base(
    class_id: i32,
    message_id: i32,
    auth: AuthCookie,
    conn: Database,
    form: &ReplyToTeacherMessageForm,
) -> Result<ClassMessageReply, AddReplyError> {
    use crate::schema::class_message_reply::dsl as class_message_reply;

    if get_user_role_in_class(auth.0, class_id, &conn)
        .await
        .is_none()
    {
        return Err(AddReplyError::PermissionError);
    }

    let contents = form.contents.clone();

    match conn
        .run(move |c| {
            diesel::insert_into(class_message_reply::class_message_reply)
                .values(NewClassMessageReply {
                    contents: &contents,
                    created_at: chrono::Utc::now().naive_utc(),
                    edited: false,
                    user_id: auth.0,
                    class_id,
                    class_message_id: message_id,
                })
                .returning(crate::schema::class_message_reply::all_columns)
                .get_result(c)
        })
        .await
    {
        Ok(class_message) => Ok(class_message),
        Err(e) => {
            error!("Error adding class message reply to the database: {:#?}", e);
            Err(AddReplyError::DatabaseError)
        }
    }
}

#[post("/<class_id>/message/<message_id>/reply", data = "<form>")]
pub async fn html_reply_to_teacher_message(
    class_id: i32,
    message_id: i32,
    auth: AuthCookie,
    conn: Database,
    form: rocket::request::Form<ReplyToTeacherMessageForm>,
) -> HtmlOrRedirect {
    match add_reply_to_teacher_message_base(class_id, message_id, auth, conn, &form).await {
        Ok(_) => HtmlOrRedirect::Redirect(Redirect::to(format!(
            "/class/{}/message/{}/view",
            class_id, message_id
        ))),
        Err(e) => match e {
            AddReplyError::PermissionError => HtmlOrRedirect::Html(permission_error()),
            AddReplyError::DatabaseError => HtmlOrRedirect::Html(database_error()),
        },
    }
}

#[post("/<class_id>/message/<message_id>/reply", data = "<form>")]
pub async fn api_reply_to_teacher_message(
    class_id: i32,
    message_id: i32,
    auth: AuthCookie,
    conn: Database,
    form: rocket::request::Form<ReplyToTeacherMessageForm>,
) -> Json<ApiResponse<ClassMessageReply>> {
    Json(
        match add_reply_to_teacher_message_base(class_id, message_id, auth, conn, &form).await {
            Ok(reply) => ApiResponse::new_ok(reply),
            Err(e) => ApiResponse::new_err(match e {
                AddReplyError::PermissionError => "invalid permissions",
                AddReplyError::DatabaseError => "database error",
            }),
        },
    )
}
