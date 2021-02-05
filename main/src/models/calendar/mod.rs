use crate::schema::caldav;
use crate::schema::caldav_unauthenticated;
use crate::schema::calendar;
use crate::schema::google_calendar;

#[derive(Debug, Queryable, Identifiable)]
#[table_name = "calendar"]
pub struct Calendar {
    pub id: i32,
    pub calendar_type: i32,
    pub user_id: i32,
}

#[derive(Debug, Insertable)]
#[table_name = "calendar"]
pub struct NewCalendar {
    pub calendar_type: i32,
    pub user_id: i32,
}

#[derive(Debug, Copy, Clone)]
pub enum CalendarType {
    GoogleCalendar,
    CalDAV,
    CalDAVUnauthenticated,
}

impl From<CalendarType> for i32 {
    fn from(ty: CalendarType) -> Self {
        match ty {
            CalendarType::GoogleCalendar => 0,
            CalendarType::CalDAV => 1,
            CalendarType::CalDAVUnauthenticated => 2,
        }
    }
}

/// Parse the user's calendar.
pub fn parse_calendar_type(ty: i32) -> CalendarType {
    match ty {
        0 => CalendarType::GoogleCalendar,
        1 => CalendarType::CalDAV,
        2 => CalendarType::CalDAVUnauthenticated,
        _ => panic!(),
    }
}

#[derive(Debug, Queryable, Identifiable)]
#[table_name = "google_calendar"]
pub struct GoogleCalendar {
    pub id: i32,
    pub calendar_id: i32,
    pub refresh_token: String,
    pub access_token: String,
    pub lovelace_calendar_id: String,
}

#[derive(Debug, Insertable)]
#[table_name = "google_calendar"]
pub struct NewGoogleCalendar<'a> {
    pub refresh_token: &'a str,
    pub access_token: &'a str,
    pub calendar_id: i32,
    pub lovelace_calendar_id: &'a str,
}

#[derive(Queryable, Identifiable, Debug)]
#[table_name = "caldav"]
pub struct CalDav {
    pub id: i32,
    pub calendar_id: i32,
    pub username: String,
    pub password: String,
    pub url: String,
}

#[derive(Insertable, Debug)]
#[table_name = "caldav"]
pub struct NewCalDav<'a> {
    pub calendar_id: i32,
    pub username: &'a str,
    pub password: &'a str,
    pub url: &'a str,
}

#[derive(Queryable, Identifiable, Debug)]
#[table_name = "caldav_unauthenticated"]
pub struct CalDAVUnauthenticated {
    pub id: i32,
    pub calendar_id: i32,
    pub url: String,
}

#[derive(Insertable, Debug)]
#[table_name = "caldav_unauthenticated"]
pub struct NewCalDAVUnauthenticated<'a> {
    pub calendar_id: i32,
    pub url: &'a str,
}
