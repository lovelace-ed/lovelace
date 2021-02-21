use crate::models::institution::{administrator::NewAdministrator, NewInstitution};
use chrono::Utc;
use diesel::prelude::*;
use malvolio::prelude::*;
use mercutio::Apply;
use portia::{form::FormStyle, levels::Level, render::Render};
use rocket::FromForm;
use rocket_contrib::json::Json;

use crate::{
    auth::AuthCookie,
    db::Database,
    models::institution::Institution,
    schema::{administrator, institution},
    utils::{default_head, error::LovelaceError, json_response::ApiResponse},
};

fn new_institution_form() -> Form {
    Form::new()
        .apply(FormStyle)
        .child(
            Input::new()
                .attribute(Type::Text)
                .attribute(Placeholder::new("Institution name"))
                .attribute(Name::new("name")),
        )
        .child(
            Input::new()
                .attribute(Type::Text)
                .attribute(Type::Text)
                .attribute(Placeholder::new(
                    "Institution website – leave this blank if you don't have one",
                ))
                .attribute(Name::new("website")),
        )
}

#[get("/register")]
pub async fn register_new_institution_page(_auth: AuthCookie) -> Html {
    Html::new()
        .head(default_head("Register a new institution."))
        .body(
            Body::new().child(
                Level::new()
                    .child(H1::new("Register a new institution"))
                    .child(new_institution_form()),
            ),
        )
}

#[derive(FromForm, Serialize, Deserialize, Debug)]
pub struct CreateNewInstitutionForm {
    name: String,
    domain: String,
}

async fn register_new_institution(
    auth: AuthCookie,
    conn: Database,
    data: &CreateNewInstitutionForm,
) -> Result<Institution, LovelaceError> {
    let name = data.name.clone();
    let domain = data.domain.clone();
    conn.run(move |c| {
        diesel::insert_into(institution::table)
            .values(NewInstitution {
                name: &name,
                domain: &domain,
                created: Utc::now().naive_utc(),
                enforce_same_domain: false,
            })
            .returning(institution::all_columns)
            .get_result::<Institution>(c)
            .map(|res| {
                diesel::insert_into(administrator::table)
                    .values(NewAdministrator {
                        user_id: auth.0,
                        institution_id: res.id,
                    })
                    .execute(c)
                    .map_err(|e| {
                        error!("{:#?}", e);
                        LovelaceError::DatabaseError
                    })
                    .map(|_| res)
            })
            .map_err(|e| {
                error!("{:#?}", e);
                LovelaceError::DatabaseError
            })
    })
    .await?
}

#[post("/register", data = "<form>")]
pub async fn html_register_new_institution(
    auth: AuthCookie,
    form: rocket::request::Form<CreateNewInstitutionForm>,
    conn: Database,
) -> Html {
    match register_new_institution(auth, conn, &form).await {
        Ok(institution) => Html::new()
            .status(200)
            .head(default_head("Successfully registered."))
            .body(
                Body::new()
                    .child(Level::new().child(H1::new("Successfully registered.")))
                    .child(
                        A::new()
                            .attribute(Href::new(format!("/institution/{}", institution.id)))
                            .text("View the overview of this institution."),
                    ),
            ),
        Err(e) => e.render(),
    }
}

#[post("/register", data = "<form>")]
pub async fn api_register_new_institution(
    auth: AuthCookie,
    form: Json<CreateNewInstitutionForm>,
    conn: Database,
) -> Json<ApiResponse<Institution>> {
    Json(match register_new_institution(auth, conn, &form).await {
        Ok(institution) => ApiResponse::new_ok(institution),
        Err(e) => From::from(e),
    })
}

#[cfg(test)]
mod test_register_institution {
    use crate::utils::login_user;
    use bcrypt::DEFAULT_COST;
    use chrono::Utc;
    use diesel::prelude::*;
    use rocket::http::ContentType;

    use crate::{
        db::Database,
        models::{institution::Institution, NewUser, User},
        schema::{administrator, institution, users},
        utils::client,
    };
    const USERNAME: &str = "admin";
    const EMAIL: &str = "admin@example.com";
    const PASSWORD: &str = "s3cuRE_passw-rd";
    const TIMEZONE: &str = "Africa/Abidjan";
    const NAME: &str = "Some educational institution";
    const WEBSITE: &str = "https://example.com";

    async fn setup_env(conn: Database) {
        conn.run(|c| {
            diesel::insert_into(users::table)
                .values(NewUser {
                    username: USERNAME,
                    email: EMAIL,
                    password: &bcrypt::hash(PASSWORD, DEFAULT_COST).unwrap(),
                    created: Utc::now().naive_utc(),
                    email_verified: true,
                    timezone: TIMEZONE,
                })
                .execute(c)
                .unwrap();
        })
        .await
    }

    #[rocket::async_test]
    async fn test_can_create_institution() {
        let client = client().await;
        setup_env(Database::get_one(client.rocket()).await.unwrap()).await;
        login_user(USERNAME, PASSWORD, &client).await;
        // create institution
        let res = client
            .post("/institution/register")
            .header(ContentType::Form)
            .body(format!("name={}&domain={}", NAME, WEBSITE))
            .dispatch()
            .await;
        let string = res.into_string().await.expect("invalid body string");
        println!("{}", string);
        assert!(string.contains("registered"));

        // check database state
        Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(|c| {
                let institution = institution::table
                    .filter(institution::name.eq(NAME))
                    .filter(institution::domain.eq(WEBSITE))
                    .first::<Institution>(c)
                    .expect("failed to find institution");
                assert_eq!(institution.name, NAME);
                assert_eq!(institution.domain, WEBSITE);
                administrator::table
                    .inner_join(users::table)
                    .filter(users::email.eq(EMAIL))
                    .filter(users::username.eq(USERNAME))
                    .select(users::all_columns)
                    .first::<User>(c)
                    .expect("failed to find administrator");
            })
            .await;
    }
}
