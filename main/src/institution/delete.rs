use diesel::prelude::*;
use malvolio::prelude::*;
use portia::{levels::Level, render::Render};
use rocket_contrib::json::Json;

use crate::{
    auth::AuthCookie,
    db::{Database, DatabaseConnection},
    schema::{administrator, institution, users},
    utils::{
        default_head,
        error::{LovelaceError, LovelaceResult},
        json_response::ApiResponse,
    },
};

#[get("/<institution_id>/delete")]
pub async fn delete_institution_page(
    institution_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Html {
    let has_permission = conn
        .run(move |c| has_permission(institution_id, auth, c))
        .await;
    if !has_permission {
        return LovelaceError::PermissionError.render();
    }
    Html::new()
        .head(default_head("Delete this institution"))
        .body(
            Body::new().child(
                Level::new()
                    .child(H1::new("Delete this institution"))
                    .child(
                        Form::new()
                            .child(
                                Input::new()
                                    .attribute(Type::Submit)
                                    .attribute(Value::new("Delete this institution.")),
                            )
                            .child(Label::new(
                                "WARNING: this is an irreversible action!!! If you
                                      submit this form, there is no going back!!!",
                            )),
                    ),
            ),
        )
}

fn has_permission(institution_id: i32, auth: AuthCookie, c: &DatabaseConnection) -> bool {
    match diesel::select(diesel::dsl::exists(
        institution::table
            .inner_join(administrator::table.inner_join(users::table))
            .filter(users::id.eq(auth.0))
            .filter(institution::id.eq(institution_id)),
    ))
    .get_result(c)
    {
        Ok(t) => t,
        Err(e) => {
            error!("{:#?}", e);
            false
        }
    }
}

async fn apply_delete_institution(
    institution_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> LovelaceResult<()> {
    conn.run(move |c| {
        if has_permission(institution_id, auth, c) {
            diesel::delete(institution::table.filter(institution::id.eq(institution_id)))
                .execute(c)
                .map(drop)
                .map_err(|e| {
                    error!("{:#?}", e);
                    LovelaceError::DatabaseError
                })
        } else {
            Err(LovelaceError::PermissionError)
        }
    })
    .await
}

#[post("/<institution_id>/delete")]
pub async fn html_delete_institution(
    institution_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Html {
    match apply_delete_institution(institution_id, auth, conn).await {
        Ok(()) => Html::new()
            .head(default_head("Deleted that institution."))
            .body(
                Body::new().child(
                    Level::new()
                        .child(H1::new("We're sad to see you go."))
                        .child(P::with_text(
                            "Hopefully we'll meet again :). We wish you all the best.",
                        )),
                ),
            ),
        Err(e) => Html::new()
            .head(default_head("Could not carry out the operation."))
            .body(
                Body::new().child(
                    Level::new()
                        .child(H1::new("Couldn't delete that institution."))
                        .child(P::with_text(
                            "We assure you – this is not an intentional to keep you using the
                            platform. We're working to fix this as soon as possible. In the
                            meantime, please try again.",
                        ))
                        .child(Render::<Div>::render(e)),
                ),
            ),
    }
}

#[post("/<institution_id>/delete")]
pub async fn api_delete_institution(
    institution_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Json<ApiResponse<()>> {
    Json(
        match apply_delete_institution(institution_id, auth, conn).await {
            Ok(()) => ApiResponse::new_ok(()),
            Err(e) => From::from(e),
        },
    )
}

#[cfg(test)]
mod test_delete_institution {
    use bcrypt::DEFAULT_COST;
    use chrono::Utc;
    use diesel::prelude::*;

    use crate::{
        institution::{
            delete::Database,
            test_ctx::{ADMIN_PASSWORD, ADMIN_USERNAME, TIMEZONE},
        },
        models::NewUser,
        schema::users,
        utils::{client, login_user},
    };

    use super::super::test_ctx::setup_env;

    #[rocket::async_test]
    async fn test_admin_can_delete() {
        let client = client().await;
        let (_, _, _, institution_id, _) = Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(|c| setup_env(c))
            .await;
        login_user(ADMIN_USERNAME, ADMIN_PASSWORD, &client).await;
        let res = client
            .post(format!("/institution/{}/delete", institution_id))
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains("Deleted that institution"));
    }

    #[rocket::async_test]
    async fn test_not_admin_cannot_delete() {
        let client = client().await;
        let (_, _, _, institution_id, _) = Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(|c| setup_env(c))
            .await;
        Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(|c| {
                diesel::insert_into(users::table)
                    .values(NewUser {
                        username: "notadmin",
                        email: "notadmin@example.com",
                        password: &bcrypt::hash("notadminpassW0RD", DEFAULT_COST).unwrap(),
                        created: Utc::now().naive_utc(),
                        email_verified: true,
                        timezone: TIMEZONE,
                    })
                    .execute(c)
            })
            .await
            .unwrap();
        login_user("notadmin", "notadminpassW0RD", &client).await;
        let res = client
            .post(format!("/institution/{}/delete", institution_id))
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains("Couldn't delete"));
    }
}
