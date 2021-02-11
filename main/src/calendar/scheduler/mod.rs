//! Task scheduling.
//!
//! The algorithm works as follows:
//!   1. Pick out all the events which are happening over the next two weeks
//!   2. Work out all the times during which the user is busy
//!   3. Work out all the tasks that the user has
//!   4. Make sure that there is actually enough time to do all the work
//!   5. Start filling in the tasks (currently we're using a shortest-task first system)
//!
//! NOTE: (because there's only one of me and thousands of lines of code) we are currently assuming
//! that all tasks take 25 minutes, which is obviously not correct

use crate::{
    db::Database,
    models::{
        calendar::{parse_calendar_type, GoogleCalendar},
        User,
    },
    schema::{
        calendar, class, class_asynchronous_task, class_student, google_calendar,
        student_class_asynchronous_task, users,
    },
};
use crate::{
    models::{ClassAsynchronousTask, StudentClassAsynchronousTask},
};
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use prospero::{
    client::DAVClient,
    error::CalDAVError,
    event::EventPointer,
    icalendar::{Component, Event},
};
use std::{ops::Add, sync::Arc};
use thiserror::Error as ThisError;
use uuid::Uuid;

#[derive(ThisError, Debug)]
pub enum SchedulingError {
    #[error("database error")]
    DatabaseError(diesel::result::Error),
    #[error("scheduling error")]
    SchedulingError(prospero::error::CalDAVError),
}

impl From<CalDAVError> for SchedulingError {
    fn from(e: CalDAVError) -> Self {
        SchedulingError::SchedulingError(e)
    }
}

impl From<diesel::result::Error> for SchedulingError {
    fn from(e: diesel::result::Error) -> Self {
        SchedulingError::DatabaseError(e)
    }
}

struct FreeSlot {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

/// Maps use events to free time.
async fn map_user_events_to_free_time(events: Vec<EventPointer>) -> Vec<FreeSlot> {
    // this shouldn't really ever happen (because otherwise people are going to start getting
    // scheduled to do things in the middle of the night)
    if events.len() <= 0 {
        return vec![FreeSlot {
            start: Utc::now(),
            end: Utc::now() + Duration::days(14),
        }];
    }
    let pad_start = Utc::now();
    let mut free_slots = vec![FreeSlot {
        start: pad_start,
        end: events.get(0).unwrap().start_time().await.unwrap(),
    }];
    for (i, event) in events.iter().enumerate() {
        if i == (events.len() - 1) {
            break;
        }
        free_slots.push(FreeSlot {
            start: event.end_time().await.unwrap(),
            end: events.get(i + 1).unwrap().start_time().await.unwrap(),
        })
    }
    free_slots
}

/// Creates a schedule for the next two weeks.
///
/// Schedules cannot be created for people who have not connected a calendar.
pub async fn two_week_schedule(user_id: i32, conn: Arc<Database>) -> Result<(), SchedulingError> {
    let (_user, calendar) = conn
        .run(move |c| {
            users::table
                .filter(users::id.eq(user_id))
                .inner_join(calendar::table)
                .select((users::all_columns, calendar::all_columns))
                .first::<(User, crate::models::calendar::Calendar)>(c)
        })
        .await?;
    let r#type = calendar.calendar_type;
    let (lovelace_client, user_client) = match parse_calendar_type(r#type) {
        crate::models::calendar::CalendarType::GoogleCalendar => {
            let gcal = conn
                .run(move |c| {
                    google_calendar::table
                        .filter(google_calendar::id.eq(calendar.id))
                        .first::<GoogleCalendar>(c)
                })
                .await?;

            cfg_if! {
                if #[cfg(test)] {
                    let user_calendar_url = "http://localhost:8080/user/calendars/calendar".to_string();
                } else {
                    let user_calendar_url = format!("https://apidata.googleusercontent.com/caldav/v2/{}/events", gcal.calendar_id);
                }
            };

            cfg_if! {
                if #[cfg(test)] {
                    (
                        DAVClient::new_unauthenticated(
                            gcal.lovelace_calendar_id,
                        ),
                        DAVClient::new_unauthenticated(
                            user_calendar_url,
                        ),
                    )
                } else {
                    (
                        DAVClient::new_oauth(
                            gcal.lovelace_calendar_id,
                            gcal.access_token.clone(),
                            gcal.refresh_token.clone(),
                        ),
                        DAVClient::new_oauth(
                            user_calendar_url,
                            gcal.access_token.clone(),
                            gcal.refresh_token,
                        ),
                    )
                }
            }
        }
        crate::models::calendar::CalendarType::CalDAV => {
            todo!()
        }
        crate::models::calendar::CalendarType::CalDAVUnauthenticated => {
            todo!()
        }
    };

    let lovelace_controller = lovelace_client.calendar();
    let user_controller = user_client.calendar();
    let user_events = user_controller
        .date_search(
            chrono::Utc::now(),
            chrono::Utc::now() + chrono::Duration::days(14),
        )
        .await
        .unwrap();
    let set_events = lovelace_controller
        .date_search(Utc::now(), Utc::now() + Duration::days(14))
        .await?;
    let mut tasks = conn
        .run(move |c| {
            student_class_asynchronous_task::table
                .inner_join(class_student::table.inner_join(users::table))
                .inner_join(class_asynchronous_task::table)
                .filter(users::id.eq(user_id))
                .filter(class_asynchronous_task::due_date.ge(chrono::Utc::now().naive_utc()))
                .filter(
                    class_asynchronous_task::due_date
                        .le((chrono::Utc::now() + Duration::days(14)).naive_utc()),
                )
                .select((
                    class_asynchronous_task::all_columns,
                    student_class_asynchronous_task::all_columns,
                ))
                .load::<(ClassAsynchronousTask, StudentClassAsynchronousTask)>(c)
        })
        .await
        .unwrap();
    let free_slots = map_user_events_to_free_time(user_events).await;

    let mut events_to_add = vec![];

    for slot in free_slots {
        if tasks.is_empty() {
            break;
        }
        fill_slot(slot, &mut tasks, &mut events_to_add);
    }

    for event in set_events {
        event.delete().await?;
    }

    for event in events_to_add {
        lovelace_controller.save_event(event).await?;
    }

    Ok(())
}

/// Recursively occupies the provided time slot with as many events as possible.
fn fill_slot(
    slot: FreeSlot,
    tasks: &mut Vec<(ClassAsynchronousTask, StudentClassAsynchronousTask)>,
    events_to_add: &mut Vec<Event>,
) {
    if tasks.len() <= 0 {
        return;
    }
    let len = slot.end - slot.start;
    if len > Duration::minutes(25) {
        let (task, _) = tasks.pop().unwrap();
        events_to_add.push(
            Event::new()
                .uid(&Uuid::new_v4().to_string())
                .starts(slot.start)
                .ends(slot.start.add(Duration::minutes(25)))
                .summary(
                    &format!(
                        "Task title: {} Task description: {}",
                        task.title, task.description
                    )
                    .chars()
                    .map(|char| if char == '\n' { ' ' } else { char })
                    .collect::<String>()
                    .as_str(),
                )
                .done(),
        );
        fill_slot(
            FreeSlot {
                start: slot.start + Duration::minutes(25),
                end: slot.end,
            },
            tasks,
            events_to_add,
        );
    }
}

/// Computes the schedule for all users in a class.
pub async fn schedule_class(class_id: i32, conn: Arc<Database>) -> Result<(), SchedulingError> {
    match conn
        .run(move |c| {
            class::table
                .filter(class::id.eq(class_id))
                .inner_join(class_student::table.inner_join(users::table))
                .select(users::id)
                .load::<i32>(c)
        })
        .await
    {
        Ok(users) => {
            for user_id in users {
                two_week_schedule(user_id, conn.clone()).await?;
            }
            Ok(())
        }
        Err(e) => Err(SchedulingError::DatabaseError(e)),
    }
}
