use crate::{
    auth::AuthCookie,
    class::get_user_role_in_class,
    db::Database,
    models::ClassMessage,
    schema::{class_message, users},
    utils::{default_head, error_messages::database_error, json_response::ApiResponse},
};

use diesel::prelude::*;
use malvolio::prelude::*;
use portia::levels::Level;
use rocket::serde::json::Json;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum ListMessagesError {
    #[error("permission error")]
    PermissionError,
    #[error("database error")]
    DatabaseError,
}

async fn list_all_messages_base(
    id: i32,
    conn: Database,
    auth: AuthCookie,
) -> Result<(crate::models::Class, Vec<(ClassMessage, String)>), ListMessagesError> {
    use crate::schema::class::dsl as class;
    if get_user_role_in_class(auth.0, id, &conn).await.is_some() {
        let class_id = id;
        let class = match conn
            .run(move |c| {
                class::class
                    .filter(class::id.eq(class_id))
                    .first::<crate::models::Class>(c)
            })
            .await
        {
            Ok(t) => t,
            Err(_) => return Err(ListMessagesError::DatabaseError),
        };
        let class_clone = class.clone();
        let messages = match conn
            .run(move |c| {
                ClassMessage::belonging_to(&class_clone)
                    .inner_join(users::table)
                    .select((class_message::all_columns, users::username))
                    .load::<(ClassMessage, String)>(c)
            })
            .await
        {
            Ok(t) => t,
            Err(_) => return Err(ListMessagesError::DatabaseError),
        };
        Ok((class, messages))
    } else {
        Err(ListMessagesError::PermissionError)
    }
}

#[get("/<id>/message")]

pub async fn html_list_all_messages(id: i32, conn: Database, auth: AuthCookie) -> Html {
    match list_all_messages_base(id, conn, auth).await {
        Ok((class, messages)) => Html::default()
            .head(default_head(format!("Messages in class {}", class.name)))
            .body(
                Body::default()
                    .child(H1::new(format!("Messages in class {}", class.name)))
                    .child(Level::new().children(messages.into_iter().map(|message| {
                        Div::new()
                            .child(H3::new(message.0.title))
                            .child(P::with_text(message.0.contents))
                    }))),
            ),
        Err(e) => {
            match e {
                ListMessagesError::PermissionError => {
                    Html::default()
                        .head(default_head("Permission error".to_string()))
                        .body(Body::default().child(H1::new("Permission error")).child(
                            P::with_text("You don't have permission to view this class."),
                        ))
                }
                ListMessagesError::DatabaseError => database_error(),
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ClassMessageWithUsername {
    message: ClassMessage,
    username: String,
}

#[derive(Serialize, Deserialize)]
pub struct ListAllClassMessages {
    class: crate::models::Class,
    messages: Vec<ClassMessageWithUsername>,
}

#[get("/<id>/message")]
pub async fn api_list_all_messages(
    id: i32,
    conn: Database,
    auth: AuthCookie,
) -> Json<ApiResponse<ListAllClassMessages>> {
    Json(match list_all_messages_base(id, conn, auth).await {
        Ok((class, messages)) => ApiResponse::new_ok(ListAllClassMessages {
            class,
            messages: messages
                .into_iter()
                .map(|(message, username)| ClassMessageWithUsername { message, username })
                .collect(),
        }),
        Err(e) => match e {
            ListMessagesError::PermissionError => {
                ApiResponse::new_err("You don't have permission to do this.")
            }
            ListMessagesError::DatabaseError => {
                ApiResponse::new_err("We encountered a database error fulfilling this request.")
            }
        },
    })
}
