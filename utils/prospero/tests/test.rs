#[tokio::test]
#[cfg(feature = "caldav_test")]
/// Note that this assumes that a test server is running at localhost:8080
///
/// This is automatically done on our continuous integration
async fn test_caldav_calendars() {
    use chrono::{Duration, Utc};
    use icalendar::{Component, Event};
    use prospero::client::DavClient;
    use std::ops::Add;

    let client = DavClient::new_unauthenticated("http://localhost:8080/user/calendars/calendar");
    let calendar = client.calendar();
    calendar
        .save_event(
            Event::new()
                .summary("some-summary")
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
    let mut sorted = vec![];
    let first = if dates[0].start_time().await.unwrap() > dates[1].start_time().await.unwrap() {
        1
    } else {
        0
    };
    sorted.push(dates[first].clone());
    sorted.push(dates[1 - first].clone());
    assert_eq!(sorted.len(), 2);
    assert_eq!(
        sorted[0].summary().await.unwrap(),
        "some-summary".to_string()
    );
    assert_eq!(
        sorted[1].summary().await.unwrap(),
        "some-other-summary".to_string()
    );
    assert!(sorted[0].start_time().await.is_ok());
    assert!(sorted[0].end_time().await.is_ok());
    assert!(sorted[1].start_time().await.is_ok());
    assert!(sorted[1].end_time().await.is_ok());
}
