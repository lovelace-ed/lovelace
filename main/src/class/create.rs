/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

use chrono::Utc;
use diesel::prelude::*;
use malvolio::prelude::*;
use mercutio::Apply;
use portia::{
    form::{FormStyle, FormSubmitInputStyle, FormTextInputStyle},
    render::Render,
};
use rocket::serde::json::Json;

use crate::{
    auth::AuthCookie,
    db::Database,
    models::{NewClass, NewClassTeacher},
    schema::{class, class_teacher},
    utils::{
        default_head,
        error::{LovelaceError, LovelaceResult},
        json_response::ApiResponse,
    },
};

fn create_class_form() -> malvolio::prelude::Form {
    malvolio::prelude::Form::new()
        .apply(FormStyle)
        .attribute(Method::Post)
        .child(
            Input::default()
                .apply(FormTextInputStyle)
                .attribute(Type::Text)
                .attribute(Placeholder::new("Class name")),
        )
        .child(
            Input::default()
                .apply(FormTextInputStyle)
                .attribute(Type::Textarea)
                .attribute(Placeholder::new("Add a description for this class here.")),
        )
        .child(
            Input::default()
                .apply(FormSubmitInputStyle)
                .attribute(Type::Submit)
                .attribute(Value::new("Create class")),
        )
}

#[get("/class/create")]
pub fn create_class_page(_auth_cookie: AuthCookie) -> Html {
    Html::default().head(default_head("Create a class")).body(
        Body::default()
            .child(H1::new("Create a class"))
            .child(create_class_form()),
    )
}

#[derive(FromForm, Debug, Clone, Serialize, Deserialize)]
pub struct CreateClassForm {
    name: String,
    description: String,
}

async fn create_class(
    data: &CreateClassForm,
    auth: AuthCookie,
    conn: Database,
) -> LovelaceResult<crate::models::Class> {
    let data = data.clone();
    conn.run(move |c| {
        diesel::insert_into(class::table)
            .values(NewClass::new(
                &data.name,
                &data.description,
                Utc::now().naive_utc(),
                &nanoid!(5),
                None,
                None,
            ))
            .get_result::<crate::models::Class>(c)
            .map(|res| {
                diesel::insert_into(class_teacher::table)
                    .values(NewClassTeacher {
                        user_id: auth.0,
                        class_id: res.id,
                    })
                    .execute(c)
                    .map(|_| res)
                    .map_err(|e| {
                        error!("{:#?}", e);
                        LovelaceError::DatabaseError
                    })
            })
            .map_err(|e| {
                error!("{:#?}", e);
                LovelaceError::DatabaseError
            })
    })
    .await?
}

#[post("/class/create", data = "<form>")]
pub async fn html_create_class(
    form: rocket::form::Form<CreateClassForm>,
    auth: AuthCookie,
    conn: Database,
) -> Html {
    match create_class(&form, auth, conn).await {
        Ok(class) => Html::default()
            .head(default_head("Successfully created".to_string()))
            .body(
                Body::default()
                    .child(H1::new("This class has been sucessfully created"))
                    .child(
                        A::default()
                            .attribute(Href::new(format!("/class/{}", class.id)))
                            .text("Click me to the class description.".to_string()),
                    ),
            ),
        Err(e) => e.render(),
    }
}

#[post("/class/create", data = "<data>")]
pub async fn api_create_class(
    data: Json<CreateClassForm>,
    auth: AuthCookie,
    conn: Database,
) -> Json<ApiResponse<crate::models::Class>> {
    Json(match create_class(&data, auth, conn).await {
        Ok(class) => ApiResponse::new_ok(class),
        Err(e) => From::from(e),
    })
}
