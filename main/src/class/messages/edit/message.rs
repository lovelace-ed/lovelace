use diesel::prelude::*;
use malvolio::prelude::*;
use mercutio::Apply;
use portia::form::{FormStyle, FormSubmitInputStyle, FormTextInputStyle};
use rocket::response::Redirect;
use rocket_contrib::json::Json;
use thiserror::Error as ThisError;

use crate::{
    auth::AuthCookie,
    db::Database,
    models::{ClassMessage, UpdateClassMessage},
    utils::{
        default_head, error_messages::database_error, html_or_redirect::HtmlOrRedirect,
        json_response::ApiResponse,
    },
};

fn edit_message_form(msg: &ClassMessage) -> Form {
    Form::new()
        .apply(FormStyle)
        .child(
            Input::new()
                .apply(FormTextInputStyle)
                .attribute(Type::Text)
                .attribute(Name::new("title"))
                .attribute(Value::new(msg.title.clone())),
        )
        .child(
            Input::new()
                .apply(FormTextInputStyle)
                .attribute(Type::Textarea)
                .attribute(Name::new("contents"))
                .attribute(Value::new(msg.contents.clone())),
        )
        .child(
            Input::new()
                .apply(FormSubmitInputStyle)
                .attribute(Type::Submit),
        )
}

#[get("/<_class_id>/message/<message_id>/edit")]
pub async fn edit_message_page(
    _class_id: i32,
    message_id: i32,
    conn: Database,
    auth: AuthCookie,
) -> Html {
    use crate::schema::class_message::dsl as class_message;
    match conn
        .run(move |c| {
            class_message::class_message
                .filter(class_message::id.eq(message_id))
                .filter(class_message::user_id.eq(auth.0))
                .first::<ClassMessage>(c)
        })
        .await
    {
        Ok(msg) => Html::default()
            .head(default_head("Insufficient permissions"))
            .body(
                Body::default()
                    .child(H1::new("Edit this message"))
                    .child(edit_message_form(&msg)),
            ),
        Err(e) => {
            error!("{:#?}", e);
            Html::default()
                .head(default_head("Insufficient permissions"))
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

#[derive(ThisError, Debug)]
pub enum EditClassMessageError {
    #[error("database error")]
    DatabaseError,
}

pub async fn apply_message_edit_base(
    message_id: i32,
    conn: Database,
    auth: AuthCookie,
    form: &EditMessageForm,
) -> Result<ClassMessage, EditClassMessageError> {
    use crate::schema::class_message::dsl as class_message;
    let title = Some(form.title.clone());
    let contents = Some(form.contents.clone());
    match conn
        .run(move |c| {
            diesel::update(
                class_message::class_message
                    .filter(class_message::id.eq(message_id))
                    .filter(class_message::user_id.eq(auth.0)),
            )
            .set(UpdateClassMessage {
                title,
                contents,
                ..Default::default()
            })
            .returning(crate::schema::class_message::all_columns)
            .get_result::<ClassMessage>(c)
        })
        .await
    {
        Ok(t) => Ok(t),
        Err(_) => Err(EditClassMessageError::DatabaseError),
    }
}

#[derive(FromForm, Debug, Clone, Serialize, Deserialize)]
pub struct EditMessageForm {
    title: String,
    contents: String,
}

#[post("/<_class_id>/message/<message_id>/edit", data = "<form>")]
pub async fn api_apply_message_edit(
    _class_id: i32,
    message_id: i32,
    conn: Database,
    auth: AuthCookie,
    form: rocket::request::Form<EditMessageForm>,
) -> Json<ApiResponse<ClassMessage>> {
    Json(
        match apply_message_edit_base(message_id, conn, auth, &form).await {
            Ok(ok) => ApiResponse::new_ok(ok),
            Err(e) => ApiResponse::new_err(match e {
                EditClassMessageError::DatabaseError => "",
            }),
        },
    )
}

#[post("/<class_id>/message/<message_id>/edit", data = "<form>")]
pub async fn html_apply_message_edit(
    class_id: i32,
    message_id: i32,
    conn: Database,
    auth: AuthCookie,
    form: rocket::request::Form<EditMessageForm>,
) -> HtmlOrRedirect {
    match apply_message_edit_base(message_id, conn, auth, &form).await {
        Ok(_) => HtmlOrRedirect::Redirect(Redirect::to(format!(
            "/class/{}/message/{}/view",
            class_id, message_id
        ))),
        Err(_) => HtmlOrRedirect::Html(database_error()),
    }
}
