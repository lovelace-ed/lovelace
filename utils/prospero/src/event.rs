use atomic_refcell::AtomicRefCell;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use format_xml::xml;
use http::Method;
use ical::parser::ical::component::IcalEvent;
use roxmltree::Document;
use std::sync::Arc;

pub(crate) const DATETIME_FORMAT: &str = "%Y%m%dT%H%M%SZ";
pub(crate) const DELETE: &[u8] = b"DELETE";

use crate::{
    calendar::get_calendar_data,
    client::{DavClient, REPORT},
    error::{CalDavError, CalDavResult},
};

const DTSTART: &str = "DTSTART";
const DTEND: &str = "DTEND";
const SUMMARY: &str = "SUMMARY";

#[derive(Debug, Clone)]
pub enum EventPointerData {
    FetchedEvent(IcalEvent),
    CreatedEventResponse { uid: String },
}

#[derive(Debug, Clone)]
pub struct EventPointer {
    pub(crate) data: AtomicRefCell<EventPointerData>,
    pub(crate) url: Arc<String>,
    pub(crate) client: Arc<DavClient>,
}

impl EventPointer {
    /// Resolves the request and retreives the event from the server.s
    async fn resolve(&self) -> CalDavResult<IcalEvent> {
        let borrow = self.data.borrow();
        match &*borrow {
            EventPointerData::FetchedEvent(event) => Ok(event.clone()),
            EventPointerData::CreatedEventResponse { uid } => {
                let body_string = xml! {
                    <?xml version="1.0" encoding="utf-8" ?>
                    <C:calendar-query xmlns:C="urn:ietf:params:xml:ns:caldav">
                      <D:prop xmlns:D="DAV:">
                        <D:getetag/>
                        <C:calendar-data/>
                      </D:prop>
                      <C:filter>
                        <C:comp-filter name="VCALENDAR">
                          <C:comp-filter name="VEVENT">
                            <C:prop-filter name="UID">
                              <C:text-match collation="i;octet"
                              >{uid}</C:text-match>
                            </C:prop-filter>
                          </C:comp-filter>
                        </C:comp-filter>
                      </C:filter>
                    </C:calendar-query>
                }
                .to_string();
                let res = self
                    .client
                    .request(Method::from_bytes(REPORT).unwrap(), self.url.as_str())
                    .await?
                    .header("Content-Type", "application/xml; charset=\"utf-8\"")
                    .body(body_string)
                    .send()
                    .await?;
                let document = res.text().await.unwrap();
                let document = Document::parse(&document).unwrap();
                let event = get_calendar_data(document.descendants())
                    .map(|e| e.events)
                    .flatten()
                    .next()
                    .map(Ok)
                    .unwrap_or(Err(CalDavError::OtherError))?;
                std::mem::drop(borrow);
                let mut d = self.data.borrow_mut();
                *d = EventPointerData::FetchedEvent(event.clone());
                std::mem::drop(d);
                Ok(event)
            }
        }
    }

    /// Refreshes the event (by sending a request to the server.)
    pub async fn refresh(&self) -> CalDavResult<()> {
        let borrow = self.data.borrow();
        let data = match &*borrow {
            EventPointerData::FetchedEvent(event) => EventPointerData::CreatedEventResponse {
                uid: event
                    .properties
                    .iter()
                    .find(|p| p.name == "UID")
                    .map(Ok)
                    .unwrap_or(Err(CalDavError::OtherError))?
                    .value
                    .clone()
                    .map(Ok)
                    .unwrap_or(Err(CalDavError::OtherError))?,
            },
            EventPointerData::CreatedEventResponse { uid } => {
                EventPointerData::CreatedEventResponse { uid: uid.clone() }
            }
        };
        std::mem::drop(borrow);
        let mut d = self.data.borrow_mut();
        *d = data;
        Ok(())
    }

    /// Returns the start time of the event.
    pub async fn start_time(&self) -> CalDavResult<DateTime<Utc>> {
        self.resolve()
            .await?
            .properties
            .iter()
            .find(|prop| prop.name == DTSTART)
            .map(|prop| prop.value.as_ref())
            .flatten()
            .map(|t| parse_date(t).map_err(|_| CalDavError::OtherError))
            .unwrap_or(Err(CalDavError::OtherError))
    }
    /// Returns the finish time of the event.
    pub async fn end_time(&self) -> CalDavResult<DateTime<Utc>> {
        self.resolve()
            .await?
            .properties
            .iter()
            .find(|prop| prop.name == DTEND)
            .map(|prop| prop.value.as_ref())
            .flatten()
            .map(|t| parse_date(t).map_err(|_| CalDavError::OtherError))
            .unwrap_or(Err(CalDavError::OtherError))
    }
    /// Returns the summary of this event.
    pub async fn summary(&self) -> CalDavResult<String> {
        self.resolve()
            .await?
            .properties
            .iter()
            .find(|prop| prop.name == SUMMARY)
            .map(|prop| prop.value.clone())
            .flatten()
            .map(Ok)
            .unwrap_or(Err(CalDavError::OtherError))
    }
    pub async fn delete(self) -> CalDavResult<()> {
        let borrow = self.data.borrow();
        let uid = match &*borrow {
            EventPointerData::FetchedEvent(event) => event
                .properties
                .iter()
                .find(|p| p.name == "UID")
                .map(Ok)
                .unwrap_or(Err(CalDavError::OtherError))?
                .value
                .clone()
                .map(Ok)
                .unwrap_or(Err(CalDavError::OtherError))?,
            EventPointerData::CreatedEventResponse { uid } => uid.clone(),
        };
        std::mem::drop(borrow);
        self.client
            .request(
                Method::from_bytes(DELETE).unwrap(),
                format!("{}/{}.ics", self.url, uid),
            )
            .await?
            .send()
            .await?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum ParseCalendarEventError {
    #[error("couldn't parse")]
    CouldntParse,
}

fn parse_date<T>(date: T) -> Result<DateTime<Utc>, ParseCalendarEventError>
where
    T: AsRef<str>,
{
    let naive_date = NaiveDateTime::parse_from_str(date.as_ref(), DATETIME_FORMAT)
        .map_err(|_| ParseCalendarEventError::CouldntParse)?;
    Ok(Utc.from_utc_datetime(&naive_date))
}
