use crate::utils::form::{FormErrorMsg, FormProducer};
use diesel::prelude::*;
use malvolio::prelude::*;
use portia::{levels::Level, render::Render};
use rocket::serde::json::Json;
use rocket::FromForm;

use crate::{
    auth::AuthCookie,
    db::Database,
    models::institution::{Institution, UpdateInstitution},
    utils::{default_head, error::LovelaceResult, json_response::ApiResponse},
};
use crate::{
    schema::{administrator, institution, users},
    utils::error::LovelaceError,
};

#[derive(FromForm, Serialize, Deserialize)]
pub struct ConfigureInstitutionForm {
    name: Option<String>,
    domain: Option<String>,
    enforce_same_domain: Option<bool>,
}

async fn apply_configure_institution(
    conn: Database,
    data: &ConfigureInstitutionForm,
    auth: AuthCookie,
    institution_id: i32,
) -> LovelaceResult<Institution> {
    let is_admin = conn
        .run(move |c| {
            diesel::select(diesel::dsl::exists(
                institution::table
                    .filter(institution::id.eq(institution_id))
                    .inner_join(administrator::table.inner_join(users::table))
                    .filter(users::id.eq(auth.0)),
            ))
            .get_result::<bool>(c)
        })
        .await
        .map_err(|e| {
            error!("{:#?}", e);
            LovelaceError::DatabaseError
        })?;
    if !is_admin {
        return Err(LovelaceError::PermissionError);
    }
    let name = data.name.clone();
    let domain = data.domain.clone();
    let enforce_same_domain = data.enforce_same_domain;
    let res = conn
        .run(move |c| {
            diesel::update(institution::table.filter(institution::id.eq(institution_id)))
                .set(UpdateInstitution {
                    name,
                    domain,
                    created: None,
                    enforce_same_domain,
                })
                .returning(institution::all_columns)
                .get_result::<Institution>(c)
        })
        .await
        .map_err(|e| {
            error!("{:#?}", e);
            LovelaceError::DatabaseError
        })?;
    Ok(res)
}

struct ConfigureInstitutionFormProducer(String, String, bool);

impl FormProducer for ConfigureInstitutionFormProducer {
    fn produce(self) -> Form {
        let Self(name, domain, enforce_same_domain) = self;
        Form::new()
            .child(
                Input::new()
                    .attribute(Type::Text)
                    .attribute(Name::new("name"))
                    .attribute(Placeholder::new(
                        "Configure the provided name of your institution.",
                    ))
                    .attribute(Value::new(name)),
            )
            .child(
                Input::new()
                    .attribute(Type::Text)
                    .attribute(Name::new("domain"))
                    .attribute(Value::new(domain)),
            )
            .child(
                Input::new()
                    .attribute(Name::new("enforce_same_domain"))
                    .attribute(Type::Checkbox)
                    .attribute(Value::new(enforce_same_domain.to_string())),
            )
    }
}

#[get("/<institution_id>/configure")]
pub async fn configure_institution_page(
    institution_id: i32,
    conn: Database,
    auth: AuthCookie,
) -> Html {
    let institution = match conn
        .run(move |c| {
            institution::table
                .filter(institution::id.eq(institution_id))
                .inner_join(administrator::table.inner_join(users::table))
                .filter(users::id.eq(auth.0))
                .select(institution::all_columns)
                .first::<Institution>(c)
        })
        .await
    {
        Ok(t) => t,
        Err(_) => return LovelaceError::DatabaseError.render(),
    };
    Html::new()
        .head(default_head("Configure institution"))
        .body(
            Body::new()
                .child(Level::new().child(H1::new("Configure new institution")))
                .child(
                    ConfigureInstitutionFormProducer(
                        institution.name,
                        institution.domain,
                        institution.enforce_same_domain,
                    )
                    .produce(),
                ),
        )
}

impl Render<Div> for Institution {
    fn render(self) -> Div {
        Level::new()
            .child(H3::new(format!("Institution: {}", self.name)))
            .child(P::with_text(if self.enforce_same_domain {
                "Same domain policy: enabled. This means that users registered with an email that \
                does not belong to your institution's domain cannot join (even if invited, unless \
                you turn this off)."
            } else {
                "Same domain policy: disabled. This means that users registered with an email that \
                does not belong to your institution's domain may join (given that they have an \
                invite)."
            }))
            .into_div()
    }
}

#[post("/<institution_id>/configure", data = "<form>")]
pub async fn html_configure_institution(
    institution_id: i32,
    conn: Database,
    auth: AuthCookie,
    form: rocket::form::Form<ConfigureInstitutionForm>,
) -> Html {
    match apply_configure_institution(conn, &form, auth, institution_id).await {
        Ok(institution) => Html::new().head(default_head("Succesfully updated")).body(
            Body::new().child(
                Level::new()
                    .child(H1::new("Succesfully updated"))
                    .child(Render::<Div>::render(institution)),
            ),
        ),
        Err(e) => FormErrorMsg(
            e,
            ConfigureInstitutionFormProducer(
                form.name.clone().unwrap_or_else(|| "".to_string()),
                form.domain.clone().unwrap_or_else(|| "".to_string()),
                form.enforce_same_domain.unwrap_or(false),
            ),
        )
        .render(),
    }
}

#[post("/<institution_id>/configure", data = "<form>")]
pub async fn api_configure_institution(
    institution_id: i32,
    conn: Database,
    auth: AuthCookie,
    form: Json<ConfigureInstitutionForm>,
) -> Json<ApiResponse<Institution>> {
    Json(
        match apply_configure_institution(conn, &form, auth, institution_id).await {
            Ok(institution) => ApiResponse::new_ok(institution),
            Err(e) => From::from(e),
        },
    )
}

#[cfg(test)]
mod test_configure_institution {
    use bcrypt::DEFAULT_COST;
    use chrono::Utc;
    use diesel::prelude::*;
    use rocket::http::ContentType;

    use crate::{
        db::Database,
        institution::test_ctx::{ADMIN_EMAIL, ADMIN_PASSWORD, NAME, TIMEZONE, WEBSITE},
        models::{institution::Institution, NewUser},
        schema::{institution, users},
        utils::{client, login_user},
    };

    use super::super::test_ctx::setup_env;

    /// Returns a tuple in the form (user_id, institution_id)

    #[rocket::async_test]
    async fn test_admin_can_edit() {
        let client = client().await;
        let (_, _, _, institution_id, _) = Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(|c| setup_env(c))
            .await;
        login_user(ADMIN_EMAIL, ADMIN_PASSWORD, &client).await;
        let res = client
            .post(format!("/institution/{}/configure", institution_id))
            .header(ContentType::Form)
            .body("domain=subdomain.example.com&enforce_same_domain=false")
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        println!("{}", string);
        assert!(string.contains("updated"));
        let institution: Institution = Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(move |c| {
                institution::table
                    .filter(institution::id.eq(institution_id))
                    .get_result(c)
            })
            .await
            .expect("could not find institution");
        assert_eq!(institution.domain, "subdomain.example.com");
        assert_eq!(institution.enforce_same_domain, false);
        assert_eq!(institution.name, NAME);
    }

    #[rocket::async_test]
    async fn test_not_admin_cannot_edit() {
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
            .post(format!("/institution/{}/configure", institution_id))
            .header(ContentType::Form)
            .body("domain=subdomain.example.com&enforce_same_domain=false")
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body response");
        assert!(string.contains("don't have permission"));
        let institution: Institution = Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(move |c| {
                institution::table
                    .filter(institution::id.eq(institution_id))
                    .get_result(c)
            })
            .await
            .expect("could not find institution");
        assert_eq!(institution.domain, WEBSITE);
        assert_eq!(institution.enforce_same_domain, false);
        assert_eq!(institution.name, NAME);
    }
}
