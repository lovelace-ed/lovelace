use diesel::prelude::*;
use malvolio::prelude::*;
use mercutio::Apply;
use portia::form::{FormStyle, FormSubmitInputStyle, FormTextInputStyle};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use thiserror::Error as ThisError;

use crate::utils::default_head;
use crate::utils::error_messages::database_error;
use crate::utils::html_or_redirect::HtmlOrRedirect;
use crate::{auth::AuthCookie, db::Database};
use crate::{
    models::{ClassMessage, NewClassMessage},
    utils::json_response::ApiResponse,
};

fn create_new_message_form() -> Form {
    Form::new()
        .apply(FormStyle)
        .attribute(Method::Post)
        .child(
            Input::new()
                .apply(FormTextInputStyle)
                .attribute(Type::Text)
                .attribute(Name::new("title")),
        )
        .child(
            Input::new()
                .apply(FormTextInputStyle)
                .attribute(Type::Textarea)
                .attribute(Name::new("contents")),
        )
        .child(
            Input::new()
                .apply(FormSubmitInputStyle)
                .attribute(Type::Submit),
        )
}

#[get("/<id>/message/new")]
/// Returns the form which needs to be filled in and submitted in order to create a new user in a
/// class.
pub async fn create_new_class_message_page(id: i32, auth: AuthCookie, conn: Database) -> Html {
    use crate::schema::class::dsl as class;
    use crate::schema::class_teacher::dsl as class_teacher;
    match conn
        .run(move |c| {
            class_teacher::class_teacher
                .filter(class_teacher::user_id.eq(auth.0))
                .filter(class_teacher::class_id.eq(id))
                .inner_join(class::class)
                .select(crate::schema::class::all_columns)
                .get_result::<crate::models::Class>(c)
        })
        .await
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

#[derive(ThisError, Debug)]
pub enum CreateNewClassMessageError {
    #[error("permission error")]
    PermissionError,
    #[error("database error")]
    DatabaseError,
}

pub async fn create_new_class_message_base(
    class_id: i32,
    auth: AuthCookie,
    conn: Database,
    form: &CreateNewMessageForm,
) -> Result<ClassMessage, CreateNewClassMessageError> {
    use crate::schema::class::dsl as class;
    use crate::schema::class_message::dsl as class_message;
    use crate::schema::class_teacher::dsl as class_teacher;
    match conn
        .run(move |c| {
            class_teacher::class_teacher
                .filter(class_teacher::user_id.eq(auth.0))
                .filter(class_teacher::class_id.eq(class_id))
                .inner_join(class::class)
                .select(crate::schema::class::all_columns)
                .get_result::<crate::models::Class>(c)
        })
        .await
    {
        Ok(_) => (),
        Err(diesel::result::Error::NotFound) => {
            return Err(CreateNewClassMessageError::PermissionError);
        }
        Err(e) => {
            error!(
                "Error running query to see if a user is in the given class: {:#?}",
                e
            );
            return Err(CreateNewClassMessageError::DatabaseError);
        }
    };
    let title = form.title.clone();
    let contents = form.contents.clone();
    match conn
        .run(move |c| {
            diesel::insert_into(class_message::class_message)
                .values(NewClassMessage {
                    title: &title,
                    contents: &contents,
                    created_at: chrono::Utc::now().naive_utc(),
                    user_id: auth.0,
                    class_id,
                    edited: false,
                })
                .returning(crate::schema::class_message::all_columns)
                .get_result::<ClassMessage>(c)
        })
        .await
    {
        Ok(class_message) => Ok(class_message),
        Err(e) => {
            error!("Error creating class page: {:#?}", e);
            Err(CreateNewClassMessageError::DatabaseError)
        }
    }
}

#[derive(FromForm, Debug, Clone, Serialize, Deserialize)]
pub struct CreateNewMessageForm {
    title: String,
    contents: String,
}

#[post("/<class_id>/message/new", data = "<form>")]
pub async fn html_apply_create_new_class_message(
    class_id: i32,
    auth: AuthCookie,
    conn: Database,
    form: rocket::form::Form<CreateNewMessageForm>,
) -> HtmlOrRedirect {
    match create_new_class_message_base(class_id, auth, conn, &form).await {
        Ok(class_message) => HtmlOrRedirect::Redirect(Redirect::to(format!(
            "/class/{}/message/{}/view",
            class_id, class_message.id
        ))),
        Err(e) => match e {
            CreateNewClassMessageError::PermissionError => {
                HtmlOrRedirect::Html(
                    Html::default()
                        .head(default_head("Error.".to_string()))
                        .body(
                            Body::default()
                                .child(H1::new("Error completing this request"))
                                .child(P::with_text(
                                    "Either the class in question doesn't exist, or you aren't a teacher in that
                                    class.",
                                )),
                        ),
                )
            }
            CreateNewClassMessageError::DatabaseError => HtmlOrRedirect::Html(database_error()),
        },
    }
}

#[post("/<class_id>/message/new", data = "<form>")]
pub async fn api_apply_create_new_class_message(
    class_id: i32,
    auth: AuthCookie,
    conn: Database,
    form: Json<CreateNewMessageForm>,
) -> Json<ApiResponse<ClassMessage>> {
    Json(
        match create_new_class_message_base(class_id, auth, conn, &form).await {
            Ok(class_message) => ApiResponse::new_ok(class_message),
            Err(e) => match e {
                CreateNewClassMessageError::PermissionError => {
                    ApiResponse::new_err("You do not have the permissions to send that message.")
                }
                CreateNewClassMessageError::DatabaseError => ApiResponse::new_err(
                    "Encountered a database error while trying to send that message.",
                ),
            },
        },
    )
}
