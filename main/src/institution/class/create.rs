use chrono::Utc;
use diesel::prelude::*;
use malvolio::prelude::*;
use mercutio::Apply;
use portia::{
    form::{FormSubmitInputStyle, FormTextInputStyle},
    levels::Level,
    render::Render,
};
use rocket::{response::Redirect, FromForm};
use rocket_contrib::json::Json;

use crate::{
    auth::AuthCookie,
    db::Database,
    models::{
        institution::{
            administrator::Administrator, student::InstitutionStudent, student_group::StudentGroup,
            teacher::InstitutionTeacher, Institution,
        },
        NewClass,
    },
    schema::{
        administrator, class, institution, institution_student, institution_teacher, student_group,
        users,
    },
    utils::{
        default_head,
        error::{LovelaceError, LovelaceResult},
        error_messages::database_error,
        html_or_redirect::HtmlOrRedirect,
        json_response::ApiResponse,
    },
};

fn create_institution_class_form(student_groups: Vec<StudentGroup>) -> Form {
    Form::new()
        .child(Label::new(
            "Pick a student group to add this class as part of. If you don't want to add this
            class as part of a student group, select \"none\".",
        ))
        .child(
            Select::new()
                .attribute(Name::new("institution_id"))
                .child(
                    SelectOption::new()
                        .attribute(Value::new("none"))
                        .text("None"),
                )
                .children(student_groups.into_iter().map(|student_group| {
                    SelectOption::new()
                        .attribute(Value::new(student_group.id.to_string()))
                        .text(student_group.name)
                })),
        )
        .child(
            Input::new()
                .apply(FormTextInputStyle)
                .attribute(Type::Text)
                .attribute(Name::new("name"))
                .attribute(Placeholder::new("Add this class' name.")),
        )
        .child(
            Input::new()
                .apply(FormTextInputStyle)
                .attribute(Type::Text)
                .attribute(Name::new("description"))
                .attribute(Placeholder::new("Add a description to this class.")),
        )
        .child(
            Input::new()
                .attribute(Type::Submit)
                .apply(FormSubmitInputStyle),
        )
}

#[get("/class/create")]
pub async fn pick_which_institution_to_create_class_as_part_of(
    conn: Database,
    auth: AuthCookie,
) -> Html {
    let institutions = match conn
        .run(
            move |c| -> Result<Vec<Institution>, diesel::result::Error> {
                let admin = institution::table
                    .inner_join(administrator::table.inner_join(users::table))
                    .filter(users::id.eq(auth.0))
                    .select(institution::all_columns)
                    .load::<Institution>(c)?;
                let teacher = institution::table
                    .filter(institution::let_teachers_create_classes.eq(true))
                    .or_filter(institution::let_all_users_create_classes.eq(true))
                    .inner_join(institution_teacher::table.inner_join(users::table))
                    .select(institution::all_columns)
                    .load::<Institution>(c)?;
                let student = institution::table
                    .filter(institution::let_all_users_create_classes.eq(true))
                    .inner_join(institution_student::table.inner_join(users::table))
                    .select(institution::all_columns)
                    .load::<Institution>(c)?;
                Ok(admin
                    .into_iter()
                    .chain(teacher.into_iter())
                    .chain(student.into_iter())
                    .collect())
            },
        )
        .await
    {
        Ok(t) => t,
        Err(e) => {
            error!("{:#?}", e);
            return database_error();
        }
    };
    Html::new()
        .status(200)
        .head(default_head("Create a new class"))
        .body(
            Body::new().child(
                Level::new()
                    .child(H1::new("Create a new class as part of an institution"))
                    .child(
                        Level::new()
                            .child(H3::new(
                                "Note: if you want to create a class which is not part of an
                                institution you can also do that.",
                            ))
                            .child(
                                A::new()
                                    .attribute(Href::new("/class/create"))
                                    .text("Create a new stand-alone class."),
                            ),
                    )
                    .apply(|level| {
                        if institutions.is_empty() {
                            level.child(H1::new("You're not part of any institutions."))
                        } else {
                            level.children(institutions.into_iter().map(|institution| {
                                A::new()
                                    .attribute(Href::new(format!(
                                        "/institution/{}/class/create",
                                        institution.id
                                    )))
                                    .text(format!("Add a class as part of {}", institution.name))
                            }))
                        }
                    }),
            ),
        )
}

async fn get_user_institution_relationship(
    conn: &Database,
    institution_id: i32,
    auth: AuthCookie,
) -> Result<
    (
        Institution,
        Option<Administrator>,
        Option<InstitutionTeacher>,
        Option<InstitutionStudent>,
    ),
    diesel::result::Error,
> {
    conn.run(move |c| {
        institution::table
            .filter(institution::id.eq(institution_id))
            .left_join(administrator::table)
            .left_join(institution_teacher::table)
            .left_join(institution_student::table)
            .or_filter(administrator::user_id.eq(auth.0))
            .or_filter(institution_teacher::user_id.eq(auth.0))
            .or_filter(institution_student::user_id.eq(auth.0))
            .first::<(
                Institution,
                Option<Administrator>,
                Option<InstitutionTeacher>,
                Option<InstitutionStudent>,
            )>(c)
    })
    .await
}

fn check_permissions(
    (institution, administrator, teacher, student): (
        Institution,
        Option<Administrator>,
        Option<InstitutionTeacher>,
        Option<InstitutionStudent>,
    ),
) -> bool {
    // if anyone is allowed to create a class, check that the user fits in "anyone"
    (institution.let_all_users_create_classes
        && (administrator.is_some() || student.is_some() || teacher.is_some()))
    // if only teachers and admins may create a class, check that the user matches this
    || (institution.let_teachers_add_sync_tasks
        && (administrator.is_some() || student.is_some()))
    // otherwise check if we have an administrator
    || administrator.is_some()
}

#[get("/<institution_id>/class/create")]
pub async fn create_institution_class_page(
    institution_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Html {
    let student_groups = match get_user_institution_relationship(&conn, institution_id, auth).await
    {
        Ok(res) => {
            let cond = check_permissions(res);
            if cond {
                println!("COND WAS TRUE");
                match conn
                    .run(move |c| {
                        institution::table
                            .filter(institution::id.eq(institution_id))
                            .inner_join(student_group::table)
                            .select(student_group::all_columns)
                            .load::<StudentGroup>(c)
                    })
                    .await
                {
                    Ok(t) => t,
                    Err(e) => {
                        println!("b");
                        error!("{:#?}", e);
                        return LovelaceError::DatabaseError.render();
                    }
                }
            } else {
                return LovelaceError::PermissionError.render();
            }
        }
        Err(e) => {
            error!("{:#?}", e);
            return LovelaceError::DatabaseError.render();
        }
    };
    Html::new()
        .status(200)
        .head(default_head("Create a new class"))
        .body(
            Body::new().child(
                Level::new()
                    .child(H1::new("Create a new class as part of this institution"))
                    .child(create_institution_class_form(student_groups)),
            ),
        )
}

#[derive(FromForm, Debug, Serialize, Deserialize)]
pub struct CreateClassForm {
    name: String,
    description: String,
    student_group_id: Option<i32>,
}

async fn apply_create_institution_class(
    conn: Database,
    auth: AuthCookie,
    data: &CreateClassForm,
    institution_id: i32,
) -> LovelaceResult<crate::models::Class> {
    let name = data.name.clone();
    let description = data.description.clone();
    let student_group_id = data.student_group_id;
    if !get_user_institution_relationship(&conn, institution_id, auth)
        .await
        .map(check_permissions)
        .unwrap_or(false)
    {
        return Err(LovelaceError::PermissionError);
    }
    conn.run(move |c| {
        diesel::insert_into(class::table)
            .values(NewClass {
                name: &name,
                description: &description,
                created: Utc::now().naive_utc(),
                code: &nanoid!(5),
                institution_id: Some(institution_id),
                student_group_id,
            })
            .returning(class::all_columns)
            .get_result(c)
    })
    .await
    .map_err(|e| {
        error!("{:#?}", e);
        LovelaceError::DatabaseError
    })
}

#[post("/<institution_id>/class/create", data = "<data>")]
pub async fn html_create_institution_class(
    conn: Database,
    auth: AuthCookie,
    data: rocket::request::Form<CreateClassForm>,
    institution_id: i32,
) -> HtmlOrRedirect {
    match apply_create_institution_class(conn, auth, &data, institution_id).await {
        Ok(class) => HtmlOrRedirect::Redirect(Redirect::to(format!("/class/{}", class.id))),
        Err(e) => HtmlOrRedirect::Html(e.render()),
    }
}

#[post("/<institution_id>/class/create", data = "<data>")]
pub async fn api_create_institution_class(
    conn: Database,
    auth: AuthCookie,
    data: Json<CreateClassForm>,
    institution_id: i32,
) -> Json<ApiResponse<crate::models::Class>> {
    Json(
        match apply_create_institution_class(conn, auth, &data, institution_id).await {
            Ok(class) => ApiResponse::new_ok(class),
            Err(e) => From::from(e),
        },
    )
}

#[cfg(test)]
pub mod test {
    use diesel::prelude::*;
    use rocket::http::ContentType;

    use crate::{
        db::Database,
        institution::test_ctx::{setup_env, ADMIN_PASSWORD, ADMIN_USERNAME, STUDENT_GROUP_NAME},
        schema::class,
        utils::{client, login_user},
    };

    const CLASS_NAME: &str = "class-name";
    const CLASS_DESCRIPTION: &str = "class-description";

    #[rocket::async_test]
    async fn test_create_class_handling() {
        let client = client().await;
        let (_, _, _, institution_id, student_group_id) = Database::get_one(client.rocket())
            .await
            .unwrap()
            .run(|c| setup_env(c))
            .await;
        login_user(ADMIN_USERNAME, ADMIN_PASSWORD, &client).await;

        let listing = client.get("/institution/class/create").dispatch().await;
        let res = listing.into_string().await.expect("invalid body response");
        assert!(res.contains(&format!("/institution/{}", institution_id)));

        let create_specific_task = client
            .get(format!("/institution/{}/class/create", institution_id))
            .dispatch()
            .await;
        let res = create_specific_task
            .into_string()
            .await
            .expect("invalid body response");
        println!("{}", res);
        assert!(res.contains(STUDENT_GROUP_NAME));

        let res = client
            .post(format!("/institution/{}/class/create", institution_id))
            .header(ContentType::Form)
            .body(format!(
                "name={}&description={}&student_group_id={}",
                CLASS_NAME, CLASS_DESCRIPTION, student_group_id
            ))
            .dispatch()
            .await;
        assert_eq!(res.status().code, 303);
        {
            let res = Database::get_one(client.rocket())
                .await
                .unwrap()
                .run(move |c| {
                    class::table
                        .filter(class::student_group_id.eq(student_group_id))
                        .filter(class::institution_id.eq(institution_id))
                        .first::<crate::models::Class>(c)
                        .unwrap()
                })
                .await;

            assert_eq!(res.description, CLASS_DESCRIPTION);
            assert_eq!(res.name, CLASS_NAME);
        }
    }
}
