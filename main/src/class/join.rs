/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

use diesel::prelude::*;
use malvolio::prelude::*;
use rocket::serde::json::Json;

use crate::utils::default_head;
use crate::utils::error_message;
use crate::{
    auth::AuthCookie,
    db::Database,
    models::{Class, ClassStudent, NewClassStudent},
    utils::{error::LovelaceError, json_response::ApiResponse},
};

#[get("/join/<join_code>")]
pub async fn html_join_class(join_code: String, user_id: AuthCookie, conn: Database) -> Html {
    use crate::schema::class::dsl as class;
    let class_id = match conn
        .run(|c| {
            class::class
                .filter(class::code.eq(join_code))
                .first::<Class>(c)
        })
        .await
    {
        Ok(t) => t,
        Err(diesel::result::Error::NotFound) => {
            return error_message(
                "Class not found".to_string(),
                "A class with that join code cannot be found.".to_string(),
            )
        }
        Err(_) => {
            return error_message(
                "Internal server errorr".to_string(),
                "We've run into problems on our end, which we're fixing as we speak.".to_string(),
            )
        }
    };
    match conn
        .run(move |c| {
            diesel::insert_into(crate::schema::class_student::table)
                .values(NewClassStudent {
                    user_id: user_id.0,
                    class_id: class_id.id,
                })
                .get_result::<ClassStudent>(c)
        })
        .await
    {
        Ok(_) => Html::default()
            .head(default_head("Joined".to_string()))
            .body(
                Body::default()
                    .child(H1::new("Class joined!"))
                    .child(P::with_text("You have sucessfully joined this class.")),
            ),
        Err(_) => error_message(
            "Internal server error".to_string(),
            "Something's up with our database – fear not, we're fixing it.".to_string(),
        ),
    }
}

#[get("/join/<join_code>")]
pub async fn api_join_class(
    join_code: String,
    user_id: AuthCookie,
    conn: Database,
) -> Json<ApiResponse<crate::models::Class>> {
    use crate::schema::class::dsl as class;
    let class_instance = match conn
        .run(|c| {
            class::class
                .filter(class::code.eq(join_code))
                .first::<crate::models::Class>(c)
        })
        .await
    {
        Ok(t) => t,
        Err(diesel::result::Error::NotFound) => {
            return Json(ApiResponse::new_err(
                "A class with that join code could not be found.",
            ))
        }
        Err(_) => return Json(From::from(LovelaceError::DatabaseError)),
    };
    let class_id = class_instance.id;
    Json(
        match conn
            .run(move |c| {
                diesel::insert_into(crate::schema::class_student::table)
                    .values(NewClassStudent {
                        user_id: user_id.0,
                        class_id,
                    })
                    .get_result::<ClassStudent>(c)
            })
            .await
        {
            Ok(_) => ApiResponse::new_ok(class_instance),
            Err(_) => LovelaceError::DatabaseError.into(),
        },
    )
}
