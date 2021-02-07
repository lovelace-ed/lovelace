use std::ops::Add;

use chrono::{Duration, Utc};
use prospero::{client::DAVClient, event::NewEvent};

#[tokio::test]
#[cfg(feature = "caldav_test")]
/// Note that this assumes that a test server is running at localhost:8080
///
/// This is automatically done on our continuous integration
async fn test_caldav_calendars() {
    let client = DAVClient::new_unauthenticated("http://localhost:8080/user/calendars/calendar");
    let calendar = client.calendar();
    calendar
        .save_event(NewEvent::new(
            "someId",
            Utc::now().add(Duration::days(5)),
            Utc::now().add(Duration::days(15)),
            "some-summary",
        ))
        .await
        .expect("failed to add event");
    calendar
        .save_event(NewEvent::new(
            "someId2",
            Utc::now().add(Duration::days(6)),
            Utc::now().add(Duration::days(15)),
            "some-other-summary",
        ))
        .await
        .expect("failed to add event");
    let dates = calendar
        .date_search(Utc::now(), Utc::now().add(Duration::days(50)))
        .await
        .expect("failed to search for dates");
    assert_eq!(dates.len(), 2);
    assert_eq!(dates[0].summary, Some("some-summary".to_string()));
    assert_eq!(dates[1].summary, Some("some-other-summary".to_string()));
}
