//! Integration tests for calendaring.

use crate::{
    models::calendar::{Calendar, GoogleCalendar},
    schema::{calendar, google_calendar},
    utils::{launch, login_user, logout},
};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use prospero::client::DavClient;
use rocket::{http::ContentType, local::asynchronous::Client};
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

use crate::{
    db::{Database, DatabaseConnection},
    models::{NewClass, NewClassStudent, NewClassTeacher, NewUser},
};

use super::connect::gcal::StateValues;

const NEW_TASK_TITLE: &str = "new-task-title";
const NEW_TASK_DESCRIPTION: &str = "new-task-description";

const TEACHER_USERNAME: &str = "teacher";
const TEACHER_EMAIL: &str = "teacher@example.com";
const TEACHER_PASSWORD: &str = "124t345yEFERTYasdd324";

const STUDENT_USERNAME: &str = "student-username";
const STUDENT_EMAIL: &str = "student@example.com";
const STUDENT_PASSWORD: &str = "student-passwordsa13524rerewrweds@$E@$@#@";

const CLASS_NAME: &str = "task-name";
const CLASS_DESCRIPTION: &str = "class description";

const TIMEZONE: &str = "Africa/Abidjan";

/// Returns the class id, student id, teacher id
fn setup_env(conn: &DatabaseConnection) -> (i32, i32, i32) {
    use crate::schema::{class, class_student, class_teacher, users};
    let teacher_id = diesel::insert_into(users::table)
        .values(NewUser {
            username: TEACHER_USERNAME,
            email: TEACHER_EMAIL,
            password: &bcrypt::hash(TEACHER_PASSWORD, bcrypt::DEFAULT_COST).unwrap(),
            created: Utc::now().naive_utc(),
            email_verified: true,
            timezone: TIMEZONE,
        })
        .returning(users::id)
        .get_result::<i32>(conn)
        .unwrap();
    let student_id = diesel::insert_into(users::table)
        .values(NewUser {
            username: STUDENT_USERNAME,
            email: STUDENT_EMAIL,
            password: &bcrypt::hash(STUDENT_PASSWORD, bcrypt::DEFAULT_COST).unwrap(),
            created: Utc::now().naive_utc(),
            email_verified: true,
            timezone: TIMEZONE,
        })
        .returning(users::id)
        .get_result::<i32>(conn)
        .unwrap();
    let class_id = diesel::insert_into(class::table)
        .values(NewClass {
            name: CLASS_NAME,
            description: CLASS_DESCRIPTION,
            created: Utc::now().naive_utc(),
            code: "12345",
            institution_id: None,
            student_group_id: None,
        })
        .returning(class::id)
        .get_result(conn)
        .unwrap();
    diesel::insert_into(class_student::table)
        .values(NewClassStudent {
            user_id: student_id,
            class_id,
        })
        .execute(conn)
        .unwrap();
    diesel::insert_into(class_teacher::table)
        .values(NewClassTeacher {
            user_id: teacher_id,
            class_id,
        })
        .execute(conn)
        .unwrap();
    (class_id, student_id, teacher_id)
}

// todo: fix this
#[allow(unused)]
async fn unauthenticated_caldav_integration_test() {
    // setup CalDAV test server
    let client = rocket::local::asynchronous::Client::tracked(launch())
        .await
        .unwrap();
    let (_class_id, _student_id, _teacher_id) = Database::get_one(client.rocket())
        .await
        .unwrap()
        .run(|c| setup_env(c))
        .await;

    // test user can link calendar
    let add_calendar_response = client
        .post("/calendar/unauthenticated_caldav/link")
        .header(ContentType::Form)
        .dispatch()
        .await;
    let string = add_calendar_response
        .into_string()
        .await
        .expect("invalid body response");
    assert!(string.contains("connected"));
}

const AUTH_CODE: &str = "asfdget";

#[rocket::async_test]
async fn google_oauth_caldav_integration_test() {
    std::env::set_var("OAUTH_TEST_SERVER", "https://example.com");
    std::env::set_var("HOSTNAME", "https://example.com/redirect");
    std::env::set_var("CLIENT_ID", "some-var");
    std::env::set_var("CLIENT_SECRET", "some-secret-token");
    std::env::set_var("REDIRECT_URI", "https://example.com");

    let test_server = MockServer::start().await;
    std::env::set_var("TOKEN_URL", format!("{}/token", test_server.uri()));

    let client = rocket::local::asynchronous::Client::tracked(launch())
        .await
        .unwrap();

    // register oauth mock
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            {
                "access_token": "some-token",
                "refresh_token": "some-refresh-token"
            }"#,
        ))
        .mount(&test_server)
        .await;

    let (class_id, student_id, _teacher_id) = Database::get_one(client.rocket())
        .await
        .unwrap()
        .run(|c| setup_env(c))
        .await;

    login_user(STUDENT_USERNAME, STUDENT_PASSWORD, &client).await;

    let add_calendar_response = client
        .post("/calendar/gcal/link")
        .header(ContentType::Form)
        .body("url=lovelace")
        .dispatch()
        .await;
    assert_eq!(add_calendar_response.status().code, 303);

    let (state_token, _) = client
        .rocket()
        .state::<StateValues>()
        .unwrap()
        .map
        .read()
        .await
        .iter()
        .find(|(_, b)| b.user_id == student_id)
        .map(|(a, b)| (a.clone(), b.clone()))
        .unwrap();

    let inp = format!(
        "/calendar/gcal/callback?state={}&code={}",
        state_token, AUTH_CODE
    );
    let res = client.get(inp).dispatch().await;
    let string = res.into_string().await.unwrap();
    assert!(string.contains("Connected your calendar"));
    let (_, google_calendar) = Database::get_one(client.rocket())
        .await
        .unwrap()
        .run(move |c| {
            calendar::table
                .filter(calendar::user_id.eq(student_id))
                .inner_join(google_calendar::table)
                .first::<(Calendar, GoogleCalendar)>(c)
        })
        .await
        .unwrap();
    assert_eq!(&google_calendar.access_token, "some-token");
    assert_eq!(&google_calendar.refresh_token, "some-refresh-token");

    sequence(&client, class_id).await;
}

/// Runs the test sequence (adds a number of events and checks that they are added to the calendar)
async fn sequence(client: &Client, class_id: i32) {
    logout(client).await;
    login_user(TEACHER_USERNAME, TEACHER_PASSWORD, client).await;
    let res = client
        .post(format!("/class/{}/task/async/create", class_id))
        .header(ContentType::Form)
        .body(format!(
            "title={}&description={}&due_date={}",
            NEW_TASK_TITLE,
            NEW_TASK_DESCRIPTION,
            (chrono::Utc::now() + chrono::Duration::days(7))
                .naive_utc()
                .format("%Y-%m-%dT%H:%M")
                .to_string()
        ))
        .dispatch()
        .await;
    let string = res.into_string().await.expect("invalid body string");
    assert!(string.contains("created"));
    rocket::tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let client = DavClient::new_unauthenticated("http://localhost:8080/user/calendars/lovelace");
    let calendar = client.calendar();
    let results = calendar
        .date_search(
            Utc::now() - Duration::days(1),
            Utc::now() + Duration::days(14),
        )
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
}
