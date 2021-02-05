use std::ops::Add;

use chrono::{Duration, Utc};
use icalendar::{Component, Event};
use prospero::client::DAVClient;

#[tokio::test]
#[cfg(feature = "caldav_test")]
/// Note that this assumes that a test server is running at localhost:8080
///
/// This is automatically done on our continuous integration
async fn test_caldav_calendars() {
    let client = DAVClient::new_unauthenticated("http://localhost:8080/user/calendars/calendar");
    let calendar = client.calendar();
    calendar
        .save_event(
            Event::new()
                .summary("some-sumary")
                .description("a description")
                .starts(Utc::now())
                .ends(Utc::now().add(Duration::days(4)))
                .done(),
        )
        .await
        .expect("failed to add event");
    calendar
        .save_event(
            Event::new()
                .summary("some-other-summary")
                .description("a description")
                .starts(Utc::now().add(Duration::days(5)))
                .ends(Utc::now().add(Duration::days(15)))
                .done(),
        )
        .await
        .expect("failed to add event");
    let dates = calendar
        .date_search(Utc::now(), Utc::now().add(Duration::days(50)))
        .await
        .expect("failed to search for dates");
    assert_eq!(dates.len(), 2);
    assert_eq!(
        dates[0].summary().await.unwrap(),
        "some-summary".to_string()
    );
    assert_eq!(
        dates[1].summary().await.unwrap(),
        "some-other-summary".to_string()
    );
    assert!(dates[0].start_time().await.is_ok());
    assert!(dates[0].end_time().await.is_ok());
    assert!(dates[1].start_time().await.is_ok());
    assert!(dates[1].end_time().await.is_ok());
}
