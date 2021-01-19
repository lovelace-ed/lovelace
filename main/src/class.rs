/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/
use chrono::Utc;
use malvolio::{Body, Div, Html, Input, A, H1, H3, P};

use diesel::prelude::*;
use rocket::request::Form;

use crate::{
    auth::AuthCookie,
    db::{Database, DatabaseConnection},
    models::{
        Class, ClassStudent, NewClass, NewClassStudent, NewClassTeacher, NewClassTeacherInvite,
        User,
    },
    utils::{default_head, error_message},
};

fn create_class_form() -> malvolio::Form {
    malvolio::Form::default()
        .attribute("method", "post")
        .child(
            Input::default()
                .attribute("type", "text")
                .attribute("placeholder", "Class name"),
        )
        .child(
            Input::default()
                .attribute("type", "textarea")
                .attribute("placeholder", "Add a description for this class here."),
        )
        .child(
            Input::default()
                .attribute("type", "submit")
                .attribute("value", "Create class"),
        )
}

#[get("/class/create")]
pub fn create_class_page(_auth_cookie: AuthCookie) -> Html {
    Html::default()
        .head(default_head("Create a class".to_string()))
        .body(
            Body::default()
                .child(H1::new("Create a class"))
                .child(create_class_form()),
        )
}

#[derive(FromForm)]
pub struct CreateClassForm {
    name: String,
    description: String,
}

#[post("/class/create", data = "<form>")]
pub fn create_class(form: Form<CreateClassForm>, cookie: AuthCookie, conn: Database) -> Html {
    use crate::schema::class::dsl as class;
    use crate::schema::class_teacher::dsl as class_teacher;
    match diesel::insert_into(class::class)
        .values(NewClass::new(
            &form.name,
            &form.description,
            Utc::now().naive_utc(),
            &nanoid!(5),
        ))
        .get_result::<Class>(&*conn)
    {
        Ok(res) => {
            match diesel::insert_into(class_teacher::class_teacher)
                .values(NewClassTeacher {
                    user_id: cookie.0,
                    class_id: res.id,
                })
                .execute(&*conn)
            {
                Ok(_) => Html::default()
                    .head(default_head("Successfully created".to_string()))
                    .body(
                        Body::default()
                            .child(H1::new("This class has been sucessfully created"))
                            .child(
                                A::new(format!("/class/{}", res.id))
                                    .text(format!("Click me to the class description.")),
                            ),
                    ),
                Err(e) => {
                    error!("{:#?}", e);
                    Html::default()
                        .head(default_head("Internal server error".to_string()))
                        .body(
                            Body::default()
                                .child(H1::new("Internal server error"))
                                .child(P::with_text(
                                    "There was a problem on our end creating this class.",
                                )),
                        )
                }
            }
        }
        Err(err) => {
            error!("{:#?}", err);
            Html::default()
                .head(default_head("Internal server error".to_string()))
                .body(
                    Body::default()
                        .child(H1::new("Internal server error"))
                        .child(P::with_text(
                            "There was a problem on our end creating this class.",
                        )),
                )
        }
    }
}

#[get("/join/<join_code>")]
pub fn join_class(join_code: String, user_id: AuthCookie, conn: Database) -> Html {
    use crate::schema::class::dsl as class;
    let class_id = match class::class
        .filter(class::code.eq(join_code))
        .first::<Class>(&*conn)
    {
        Ok(t) => t,
        Err(diesel::result::Error::NotFound) => {
            return error_message(
                format!("Class not found"),
                format!("A class with that join code cannot be found."),
            )
        }
        Err(_) => {
            return error_message(
                format!("Internal server errorr"),
                format!("We've run into problems on our end, which we're fixing as we speak."),
            )
        }
    };
    match diesel::insert_into(crate::schema::class_student::table)
        .values(NewClassStudent {
            user_id: user_id.0,
            class_id: class_id.id,
        })
        .get_result::<ClassStudent>(&*conn)
    {
        Ok(_) => Html::default()
            .head(default_head("Joined".to_string()))
            .body(
                Body::default()
                    .child(H1::new("Class joined!"))
                    .child(P::with_text("You have sucessfully joined this class.")),
            ),
        Err(_) => error_message(
            format!("Internal server error"),
            format!("Something's up with our database – fear not, we're fixing it."),
        ),
    }
}

#[get("/class")]
pub fn view_all_classes(auth_cookie: AuthCookie, conn: Database) -> Html {
    use crate::schema::class::dsl as class;
    use crate::schema::class_student::dsl as class_student;
    use crate::schema::class_teacher::dsl as class_teacher;
    let student_classes = match class_student::class_student
        .filter(class_student::user_id.eq(auth_cookie.0))
        .inner_join(class::class)
        .select(crate::schema::class::all_columns)
        .load::<Class>(&*conn)
    {
        Ok(classes) => Div::default()
            .attribute("class", "list")
            .map(|item| {
                if !classes.is_empty() {
                    item.child(H1::new(format!("Classes I'm a student in")))
                } else {
                    item
                }
            })
            .children(classes.iter().map(|class| {
                Div::default()
                    .attribute("class", "list-item")
                    .child(H3::new(class.name.clone()))
                    .child(P::with_text(class.description.clone()))
                    .child(A::new(format!("/class/{}", class.id)))
            })),
        Err(_) => Div::default().child(P::with_text(
            "There was a database error loading this content.",
        )),
    };
    let teacher_classes = match class_teacher::class_teacher
        .filter(class_teacher::user_id.eq(auth_cookie.0))
        .inner_join(class::class)
        .select(crate::schema::class::all_columns)
        .load::<Class>(&*conn)
    {
        Ok(classes) => Div::default()
            .attribute("class", "list")
            .map(|item| {
                if !classes.is_empty() {
                    item.child(H1::new("Classes I teach"))
                } else {
                    item
                }
            })
            .children(classes.iter().map(|class| {
                Div::default()
                    .attribute("class", "list-item")
                    .child(H3::new(class.name.clone()))
                    .child(P::with_text(class.description.clone()))
                    .child(A::new(format!("/class/{}", class.id)))
            })),
        Err(_) => Div::default().child(P::with_text(
            "There was a database error loading this content.",
        )),
    };
    Html::default()
        .head(default_head("Classes".to_string()))
        .body(
            Body::default()
                .child(teacher_classes)
                .child(student_classes),
        )
}

#[get("/class/<id>")]
pub fn view_class_overview(id: usize, auth_cookie: AuthCookie, conn: Database) -> Html {
    match get_user_role_in_class(auth_cookie.0 as i32, id as i32, &conn) {
        ClassMemberRole::Student => {
            let class = Class::with_id(id as i32, conn).unwrap();
            Html::default()
                .head(default_head(class.name.to_string()))
                .body(
                    Body::default()
                        .child(H1::new(format!("Class: {}", class.name)))
                        .child(P::with_text(class.description)),
                )
        }
        ClassMemberRole::Teacher => {
            let class = Class::with_id(id as i32, conn).unwrap();
            Html::default().head(default_head(class.name.clone())).body(
                Body::default()
                    .child(H1::new(format!("Class: {}", class.name)))
                    .child(H3::new(format!(
                        "Invite people to join with the code: {}",
                        class.code
                    )))
                    .child(P::with_text(class.description).child(
                        A::new(format!("/class/{}/settings", class.id)).text(format!("Settings")),
                    )),
            )
        }
        ClassMemberRole::None => Html::default()
            .head(default_head("Invalid permission".to_string()))
            .body(
                Body::default()
                    .child(H1::new("You don't have permission to view this class."))
                    .child(P::with_text(
                        "You might need to ask your teacher for an invite code.",
                    )),
            ),
    }
}

#[get("/class/<id>/settings")]
pub fn get_class_settings(id: usize, auth_cookie: AuthCookie, conn: Database) -> Html {
    if get_user_role_in_class(auth_cookie.0 as i32, id as i32, &*conn) == ClassMemberRole::Teacher {
        Html::default()
            .head(default_head("Settings".to_string()))
            .body(
                Body::default().child(H1::new("Settings")).child(
                    Div::default()
                        .child(A::new(format!("/class/{}/delete", id)).text("Delete this class.")),
                ),
            )
    } else {
        error_message(
            "Insufficient permissions.".to_string(),
            "You need to be a teacher for this class to see it's settings.".to_string(),
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
enum ClassMemberRole {
    Teacher,
    Student,
    None,
}

fn get_user_role_in_class(user: i32, class: i32, conn: &DatabaseConnection) -> ClassMemberRole {
    use crate::schema::class_student::dsl as class_student;
    use crate::schema::class_teacher::dsl as class_teacher;
    if diesel::dsl::select(diesel::dsl::exists(
        class_student::class_student
            .filter(class_student::user_id.eq(user))
            .filter(class_student::class_id.eq(class)),
    ))
    .get_result(conn)
    .unwrap()
    {
        ClassMemberRole::Student
    } else if diesel::dsl::select(diesel::dsl::exists(
        class_teacher::class_teacher
            .filter(class_teacher::user_id.eq(user))
            .filter(class_teacher::class_id.eq(class)),
    ))
    .get_result(conn)
    .unwrap()
    {
        ClassMemberRole::Teacher
    } else {
        ClassMemberRole::None
    }
}

#[get("/class/<id>/members")]
pub fn view_class_members_page(id: usize, conn: Database, auth_cookie: AuthCookie) -> Html {
    use crate::schema::class::dsl as class;
    use crate::schema::class_student::dsl as class_student;
    use crate::schema::users::dsl as users;
    if get_user_role_in_class(auth_cookie.0 as i32, id as i32, &*conn) == ClassMemberRole::None {
        return error_message(
            format!("You don't have permission to view this class."),
            format!("You might need to ask your teacher for a code to join the class."),
        );
    };
    let students = class::class
        .filter(class::id.eq(id as i32))
        .inner_join(class_student::class_student.inner_join(users::users))
        .select(crate::schema::users::all_columns)
        .load::<User>(&*conn)
        .map(|users| {
            users.into_iter().map(|user| {
                Div::default()
                    .attribute("class", "list-item")
                    .child(H3::new(user.username))
            })
        })
        .unwrap();
    let teachers = class::class
        .filter(class::id.eq(id as i32))
        .inner_join(class_student::class_student.inner_join(users::users))
        .select(crate::schema::users::all_columns)
        .load::<User>(&*conn)
        .map(|users| {
            users.into_iter().map(|user| {
                Div::default()
                    .attribute("class", "list-item")
                    .child(H3::new(user.username))
            })
        })
        .unwrap();
    Html::default()
        .head(default_head("Class".to_string()))
        .body(
            Body::default()
                .child(Div::default().child(H3::new("Teachers")).children(teachers))
                .child(Div::default().child(H3::new("Students")).children(students)),
        )
}

fn invite_user_form() -> malvolio::Form {
    malvolio::Form::default()
        .attribute("method", "post")
        .child(
            Input::default()
                .attribute("type", "text")
                .attribute("name", "invited-user-identifier"),
        )
        .child(
            Input::default()
                .attribute("type", "submit")
                .attribute("value", "Invite teacher!"),
        )
}

#[get("/class/<_id>/invite/teacher")]
pub fn invite_teacher_page(_id: usize) -> Html {
    Html::default()
        .head(default_head("Invite teacher".to_string()))
        .body(
            Body::default()
                .child(H1::new("Invite a new teacher"))
                .child(invite_user_form()),
        )
}

#[derive(FromForm)]
pub struct InviteTeacherForm {
    identifier: String,
}

fn user_is_teacher(user_id: i32, class_id: i32, conn: &DatabaseConnection) -> bool {
    use crate::schema::class_teacher::dsl as class_teacher;
    diesel::dsl::select(diesel::dsl::exists(
        class_teacher::class_teacher
            .filter(class_teacher::user_id.eq(user_id))
            .filter(class_teacher::class_id.eq(class_id)),
    ))
    .get_result(&*conn)
    .map_err(|e| error!("{:#?}", e))
    .unwrap_or(false)
}

#[post("/class/<id>/invite/teacher", data = "<form>")]
pub fn invite_teacher(
    id: usize,
    auth_cookie: AuthCookie,
    form: Form<InviteTeacherForm>,
    conn: Database,
) -> Html {
    use crate::schema::class_teacher_invite::dsl as class_teacher_invite;
    use crate::schema::users::dsl as users;
    if !user_is_teacher(auth_cookie.0, id as i32, &*conn) {
        return Html::default().head(default_head("Permission denied".to_string())).body(
            Body::default()
                .child(H1::new("Invite a new teacher"))
                .child(P::with_text(
                    "You don't have permission to do that because you're not a teacher for this class ."
                ))
                .child(invite_user_form()),
        );
    }
    match users::users
        .filter(users::username.eq(&form.identifier))
        .or_filter(users::email.eq(&form.identifier))
        .first::<User>(&*conn)
    {
        Ok(user) => {
            match diesel::insert_into(class_teacher_invite::class_teacher_invite)
                .values(NewClassTeacherInvite {
                    inviting_user_id: auth_cookie.0,
                    invited_user_id: user.id,
                    class_id: id as i32,
                    accepted: false,
                })
                .execute(&*conn)
            {
                Ok(_) => Html::default()
                    .head(default_head("Header".to_string()))
                    .body(Body::default().child(H1::new("Successfully invited that user."))),
                Err(e) => {
                    error!("{:#?}", e);
                    error_message(format!("Database error :("),
                    format!("We've run into some problems with our database. This error has been logged and
                    we're working on fixing it."))
                }
            }
        }
        Err(diesel::result::Error::NotFound) => Html::default()
            .head(default_head("Invite a new teacher".to_string()))
            .body(
                Body::default()
                    .child(H1::new("Invite a new teacher"))
                    .child(P::with_text(
                        "A teacher with that username or email could not be found.",
                    ))
                    .child(invite_user_form()),
            ),
        Err(e) => {
            error!("{:?}", e);
            error_message(
                format!("Database error"),
                format!("Something's up with our database. We're working on fixing this."),
            )
        }
    }
}

fn delete_class_form(id: usize) -> malvolio::Form {
    malvolio::Form::default()
        .child(Input::default().attribute("type", "text"))
        .child(
            Input::default()
                .attribute("type", "hidden")
                .attribute("name", "id")
                .attribute("value", id.to_string()),
        )
        .child(Input::default().attribute("type", "submit").attribute(
            "value",
            "Delete this class (which I will never be able to get back!)",
        ))
}

#[get("/class/<id>/delete")]
pub fn delete_class_page(id: usize, _auth_cookie: AuthCookie) -> Html {
    Html::default()
        .head(default_head("Delete this class".to_string()))
        .body(
            Body::default()
                .child(H1::new(
                    "Warning – after deleting a class it will be forever gone.",
                ))
                .child(H1::new("This means that you *cannot* get it back."))
                .child(delete_class_form(id)),
        )
}

#[derive(FromForm)]
pub struct DeleteClassForm {
    id: i32,
    confirm_name: String,
}

#[post("/class/delete", data = "<form>")]
pub fn delete_class(form: Form<DeleteClassForm>, auth_cookie: AuthCookie, conn: Database) -> Html {
    use crate::schema::class::dsl as class;
    use crate::schema::class_teacher::dsl as class_teacher;
    let user_is_teacher = diesel::dsl::select(diesel::dsl::exists(
        class_teacher::class_teacher
            .filter(class_teacher::user_id.eq(auth_cookie.0 as i32))
            .filter(class_teacher::class_id.eq(form.id)),
    ))
    .get_result::<bool>(&*conn)
    .map_err(|e| {
        error!("{:#?}", e);
        e
    });
    if let Ok(is_teacher) = user_is_teacher {
        if !is_teacher {
            return Html::default().head(default_head("Permission denied".to_string())).body(
                Body::default()
                    .child(H1::new("You aren't allowed to do this!"))
                    .child(P::with_text(
                        "You don't have permission to do that because you're not a teacher for this class ."
                    ))
                    .child(delete_class_form(form.id as usize))
            );
        }
    } else {
        return Html::default()
            .head(default_head("Class not found".to_string()))
            .body(
                Body::default()
                    .child(H1::new("We can't find a class with that id"))
                    .child(P::with_text(
                        "Check that the class in question does exist and try again.",
                    ))
                    .child(delete_class_form(form.id as usize)),
            );
    }
    match diesel::delete(
        class::class
            .filter(class::name.eq(&form.confirm_name))
            .filter(class::id.eq(form.id)),
    )
    .execute(&*conn)
    {
        Ok(num_deleted) => {
            if num_deleted == 0 {
                return Html::default()
                    .head(default_head("Could not delete this class".to_string()))
                    .body(
                        Body::default()
                            .child(H1::new("Delete this class"))
                            .child(P::with_text(
                                "The name you've typed in doesn't match this class's name.",
                            ))
                            .child(delete_class_form(form.id as usize)),
                    );
            }
            Html::default()
                .head(default_head("Class deleted".to_string()))
                .body(
                    Body::default()
                        .child(H1::new("Class deleted"))
                        .child(P::with_text("That class has been sucessfully deleted.")),
                )
        }
        Err(e) => {
            error!("{:#?}", e);
            error_message(
                format!("Database error"),
                format!("We ran into a database error when trying to delete this task."),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use regex::Regex;
    use rocket::http::ContentType;

    use crate::utils::{create_user, login_user, logout};

    const TIMEZONE: &str = "Africa/Abidjan";
    const TEACHER_USERNAME: &str = "some_teacher";
    const TEACHER_EMAIL: &str = "some_teacher@example.com";
    const TEACHER_PASSWORD: &str = "somePASSW0RD123";
    const SECONDARY_TEACHER_USERNAME: &str = "some_secondary_teacher";
    const SECONDARY_TEACHER_EMAIL: &str = "some_secondary_teacher@example.com";
    const SECONDARY_TEACHER_PASSWORD: &str = "SomeSEcondARyT3@CHER";
    const STUDENT_USERNAME: &str = "student_aw";
    const STUDENT_EMAIL: &str = "student@example.com";
    const STUDENT_PASSWORD: &str = "stUD3NTP@SSW0RD";
    const CLASS_NAME: &str = "Some class name";
    const CLASS_DESCRIPTION: &str = "Some class description";
    #[test]
    fn test_class_handling() {
        let client = crate::utils::client();
        create_user(
            TEACHER_USERNAME,
            TEACHER_EMAIL,
            TIMEZONE,
            TEACHER_PASSWORD,
            &client,
        );
        create_user(
            SECONDARY_TEACHER_USERNAME,
            SECONDARY_TEACHER_EMAIL,
            TIMEZONE,
            SECONDARY_TEACHER_PASSWORD,
            &client,
        );
        create_user(
            STUDENT_USERNAME,
            STUDENT_EMAIL,
            TIMEZONE,
            STUDENT_PASSWORD,
            &client,
        );

        // test can create class
        login_user(TEACHER_USERNAME, TEACHER_PASSWORD, &client);
        let mut create_class_res = client.get("/class/create").dispatch();
        let string = create_class_res
            .body_string()
            .expect("invalid body response");
        assert!(string.contains("Create a class"));

        let mut create_class_res = client
            .post("/class/create")
            .header(ContentType::Form)
            .body(format!(
                "name={}&description={}",
                CLASS_NAME, CLASS_DESCRIPTION
            ))
            .dispatch();
        assert!(create_class_res
            .body_string()
            .expect("invalid body response")
            .contains("Successfully created"));

        // test created class shows up on teacher class list
        let mut get_class_list = client.get("/class").dispatch();
        let string = get_class_list.body_string().expect("invalid body response");
        assert!(string.contains(CLASS_NAME));

        let id = Regex::new(r#"class/(?P<id>[0-9]+)"#)
            .unwrap()
            .captures(&string)
            .unwrap()
            .name("id")
            .unwrap()
            .as_str()
            .parse::<i32>()
            .unwrap();

        // test created class overview page can be seen

        let mut class_overview_page = client.get(format!("/class/{}", id)).dispatch();
        let string = class_overview_page
            .body_string()
            .expect("invalid body string");
        assert!(string.contains(CLASS_NAME));
        assert!(string.contains(CLASS_DESCRIPTION));
        let join_code =
            Regex::new(r#"Invite people to join with the code: (?P<code>[a-zA-Z0-9~_]+)"#)
                .unwrap()
                .captures(&string)
                .unwrap()
                .name("code")
                .unwrap()
                .as_str();

        // test teacher can see settings page

        let mut settings_page = client.get(format!("/class/{}/settings", id)).dispatch();
        let string = settings_page.body_string().unwrap();
        assert!(string.contains("delete"));

        // test students cannot join classes with the incorrect code
        logout(&client);

        login_user(STUDENT_EMAIL, STUDENT_PASSWORD, &client);

        let mut invalid_join_attempt = client
            .get("/join/SOME_RANDOM_CODE_WHICH_DOES_NOT_WORK+")
            .dispatch();
        let string = invalid_join_attempt.body_string().unwrap();
        assert!(string.contains("cannot be found"));

        // test students can join class

        let mut valid_join_attempt = client.get(format!("/join/{}", join_code)).dispatch();
        let string = valid_join_attempt.body_string().unwrap();
        assert!(string.contains("joined this class"));

        // test joined classes show up on student class list

        let mut student_class_list = client.get(format!("/class")).dispatch();
        let string = student_class_list
            .body_string()
            .expect("invalid body response");
        assert!(string.contains(CLASS_NAME));

        // test students can see class overview page

        let mut class_overview_page = client.get(format!("/class/{}", id)).dispatch();
        let string = class_overview_page
            .body_string()
            .expect("invalid body response");
        assert!(string.contains(CLASS_NAME));
        assert!(!string.contains("people to join"));

        // test teacher can delete class from the settings page

        logout(&client);

        login_user(TEACHER_EMAIL, TEACHER_PASSWORD, &client);

        let mut delete_page = client.get(format!("/class/{}/delete", id)).dispatch();
        let string = delete_page.body_string().expect("invalid body response");
        assert!(string.contains("Delete this class"));

        // test can't delete class without correct name

        let mut invalid_delete_request = client
            .post(format!("/class/delete"))
            .header(ContentType::Form)
            .body(format!("id={}&confirm_name=wrong", id))
            .dispatch();
        let string = invalid_delete_request
            .body_string()
            .expect("invalid body response");
        assert!(string.contains("doesn't match"));

        // test can't delete class without correct class id

        let mut invalid_delete_request = client
            .post(format!("/class/delete"))
            .header(ContentType::Form)
            .body(format!("id={}&confirm_name={}", 100000000, CLASS_NAME))
            .dispatch();
        let string = invalid_delete_request
            .body_string()
            .expect("invalid body response");
        assert!(string.contains("Permission denied"));

        // test teacher can delete class

        let mut invalid_delete_request = client
            .post(format!("/class/delete"))
            .header(ContentType::Form)
            .body(format!("id={}&confirm_name={}", id, CLASS_NAME))
            .dispatch();
        let string = invalid_delete_request
            .body_string()
            .expect("invalid body response");
        assert!(string.contains("sucessfully deleted"));

        // test teacher can't see deleted classes

        let mut class_overview_page = client.get(format!("/client/{}", id)).dispatch();
        let string = class_overview_page
            .body_string()
            .expect("invalid body string");
        assert!(!string.contains(CLASS_NAME));
        assert!(!string.contains(CLASS_DESCRIPTION));

        // test students can't see deleted classes

        logout(&client);
        login_user(STUDENT_EMAIL, STUDENT_PASSWORD, &client);
        let mut class_overview_page = client.get(format!("/client/{}", id)).dispatch();
        let string = class_overview_page
            .body_string()
            .expect("invalid body string");
        assert!(!string.contains(CLASS_NAME));
        assert!(!string.contains(CLASS_DESCRIPTION));
    }
}
