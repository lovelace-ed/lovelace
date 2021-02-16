use diesel::prelude::*;
use malvolio::prelude::*;
use rocket_contrib::json::Json;
use thiserror::Error as ThisError;

use super::super::get_user_role_in_class;
use crate::{
    auth::AuthCookie,
    css_names::{LIST, LIST_ITEM},
    db::Database,
    utils::{default_head, error_messages::database_error},
};
use crate::{
    models::{ClassMessage, ClassMessageReply},
    utils::json_response::ApiResponse,
};

#[derive(ThisError, Debug)]
pub enum ViewMessageError {
    #[error("database error")]
    DatabaseError,
    #[error("permission error")]
    PermissionError,
}

async fn view_message_base(
    class_id: i32,
    message_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Result<(ClassMessage, Vec<(ClassMessageReply, String)>), ViewMessageError> {
    use crate::schema::class::dsl as class;
    use crate::schema::class_message::dsl as class_message;
    use crate::schema::users::dsl as users;
    let role = get_user_role_in_class(auth.0, class_id, &conn).await;
    if role.is_none() {
        return Err(ViewMessageError::PermissionError);
    }
    let message = match conn
        .run(move |c| {
            class_message::class_message
                .filter(class_message::id.eq(message_id))
                .inner_join(class::class)
                .filter(class::id.eq(class_id))
                .select(crate::schema::class_message::all_columns)
                .first::<ClassMessage>(c)
        })
        .await
    {
        Ok(message) => message,
        Err(e) => {
            error!("{:#?}", e);
            return Err(ViewMessageError::DatabaseError);
        }
    };
    let message_clone = message.clone();
    match conn
        .run(move |c| {
            ClassMessageReply::belonging_to(&message_clone)
                .inner_join(crate::schema::users::table)
                .select((
                    crate::schema::class_message_reply::all_columns,
                    users::username,
                ))
                .load::<(ClassMessageReply, String)>(c)
        })
        .await
    {
        Ok(t) => Ok((message, t)),
        Err(e) => {
            error!("{:#?}", e);
            return Err(ViewMessageError::DatabaseError);
        }
    }
}

#[get("/<class_id>/message/<message_id>/view")]
pub async fn view_message(
    class_id: i32,
    message_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Html {
    match view_message_base(class_id, message_id, auth, conn).await {
        Ok((_message, replies)) => Html::default().head(default_head("".to_string())).body(
            Body::default().child(
                Div::new()
                    .attribute(malvolio::prelude::Class::from(LIST))
                    .children(replies.into_iter().map(|(reply, username)| {
                        Div::new()
                            .attribute(malvolio::prelude::Class::from(LIST_ITEM))
                            .child(H3::new(format!("Reply from {}", username)))
                            .child(P::with_text(format!(
                                "This reply was posted at {}",
                                reply.created_at.to_string()
                            )))
                            .child(P::with_text(reply.contents))
                    })),
            ),
        ),
        Err(e) => match e {
            ViewMessageError::DatabaseError => database_error(),
            ViewMessageError::PermissionError => Html::default()
                .head(default_head(
                    "You don't have permission to view this message.".to_string(),
                ))
                .body(
                    Body::default()
                        .child(H1::new("You don't have permission to view this message"))
                        .child(P::with_text(
                            "You might need to ask your teacher for an invite to this class.",
                        )),
                ),
        },
    }
}

#[derive(Serialize, Deserialize)]
pub struct Reply {
    username: String,
    reply: ClassMessageReply,
}

#[derive(Serialize, Deserialize)]
pub struct ViewMessageResponse {
    message: ClassMessage,
    replies: Vec<Reply>,
}

#[get("/<class_id>/message/<message_id>/view")]
pub async fn api_view_message(
    class_id: i32,
    message_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Json<ApiResponse<ViewMessageResponse>> {
    Json(
        match view_message_base(class_id, message_id, auth, conn).await {
            Ok((message, replies)) => ApiResponse::new_ok(ViewMessageResponse {
                message,
                replies: replies
                    .into_iter()
                    .map(|(reply, username)| Reply { username, reply })
                    .collect(),
            }),
            Err(e) => ApiResponse::new_err(match e {
                ViewMessageError::DatabaseError => {
                    "Encountered a database error trying to fulfill this message."
                }
                ViewMessageError::PermissionError => {
                    "You don't have permission to view this message."
                }
            }),
        },
    )
}
