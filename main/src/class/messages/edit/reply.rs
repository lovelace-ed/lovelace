use diesel::prelude::*;
use malvolio::prelude::*;
use mercutio::Apply;
use portia::form::{FormStyle, FormSubmitInputStyle, FormTextInputStyle};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use thiserror::Error as ThisError;

use crate::{
    auth::AuthCookie,
    db::Database,
    models::{ClassMessageReply, UpdateClassMessageReply},
    utils::{
        default_head, error_messages::database_error, html_or_redirect::HtmlOrRedirect,
        json_response::ApiResponse,
    },
};

fn edit_message_reply_form(msg: &ClassMessageReply) -> Form {
    Form::new()
        .apply(FormStyle)
        .child(
            Input::new()
                .apply(FormTextInputStyle)
                .attribute(Type::Text)
                .attribute(Value::new(msg.contents.clone())),
        )
        .child(
            Input::new()
                .apply(FormSubmitInputStyle)
                .attribute(Type::Submit),
        )
}

#[derive(ThisError, Debug)]
enum EditMessageReplyError {
    #[error("database error")]
    DatabaseError,
}

#[get("/<class_id>/message/<_message_id>/reply/<message_reply_id>/edit")]
/// This _message_id is here as a placeholder (it is later used when returning a redirect after POST
/// request to the same location)
pub async fn edit_message_reply(
    class_id: i32,
    message_reply_id: i32,
    _message_id: i32,
    conn: Database,
    auth: AuthCookie,
) -> Html {
    use crate::schema::class_message_reply::dsl as class_message_reply;
    match conn
        .run(move |c| {
            class_message_reply::class_message_reply
                .filter(class_message_reply::user_id.eq(auth.0))
                .filter(class_message_reply::class_id.eq(class_id))
                .filter(class_message_reply::id.eq(message_reply_id))
                .first::<ClassMessageReply>(c)
        })
        .await
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

async fn apply_message_reply_edit_base(
    class_id: i32,
    message_reply_id: i32,
    conn: Database,
    auth: AuthCookie,
    form: &ApplyMessageReplyEditForm,
) -> Result<ClassMessageReply, EditMessageReplyError> {
    use crate::schema::class_message_reply::dsl as class_message_reply;
    let contents = Some(form.contents.clone());
    conn.run(move |c| {
        diesel::update(
            class_message_reply::class_message_reply
                .filter(class_message_reply::id.eq(message_reply_id))
                .filter(class_message_reply::class_id.eq(class_id))
                .filter(class_message_reply::user_id.eq(auth.0)),
        )
        .set(UpdateClassMessageReply {
            contents,
            ..Default::default()
        })
        .returning(crate::schema::class_message_reply::all_columns)
        .get_result::<ClassMessageReply>(c)
    })
    .await
    .map_err(|e| {
        error!("Error updating a reply to a message: {:#?}", e);
        EditMessageReplyError::DatabaseError
    })
}

#[derive(FromForm, Debug, Clone, Serialize, Deserialize)]
pub struct ApplyMessageReplyEditForm {
    contents: String,
}

#[post(
    "/<class_id>/message/<message_id>/reply/<message_reply_id>/edit",
    data = "<form>"
)]
pub async fn html_apply_message_reply_edit(
    class_id: i32,
    message_reply_id: i32,
    message_id: i32,
    conn: Database,
    auth: AuthCookie,
    form: rocket::form::Form<ApplyMessageReplyEditForm>,
) -> HtmlOrRedirect {
    match apply_message_reply_edit_base(class_id, message_reply_id, conn, auth, &form).await {
        Ok(_) => HtmlOrRedirect::Redirect(Redirect::to(format!(
            "/class/{}/message/{}/view",
            class_id, message_id
        ))),
        Err(_) => HtmlOrRedirect::Html(database_error()),
    }
}

#[post(
    "/<class_id>/message/<_message_id>/reply/<message_reply_id>/edit",
    data = "<form>"
)]
pub async fn api_apply_message_reply_edit(
    class_id: i32,
    message_reply_id: i32,
    _message_id: i32,
    conn: Database,
    auth: AuthCookie,
    form: Json<ApplyMessageReplyEditForm>,
) -> Json<ApiResponse<ClassMessageReply>> {
    Json(
        match apply_message_reply_edit_base(class_id, message_reply_id, conn, auth, &form).await {
            Ok(reply) => ApiResponse::new_ok(reply),
            Err(e) => ApiResponse::new_err(match e {
                EditMessageReplyError::DatabaseError => {
                    "Encountered a database error when trying to edit that message."
                }
            }),
        },
    )
}
