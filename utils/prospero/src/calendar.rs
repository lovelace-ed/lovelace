use std::sync::Arc;

use atomic_refcell::AtomicRefCell;

use crate::{
    client::{DAVClient, REPORT},
    error::{CalDAVError, CalDAVResult},
    event::{EventPointer, EventPointerData, DATETIME_FORMAT},
};
use chrono::{DateTime, Utc};
use ical::parser::ical::component::IcalCalendar;
use icalendar::Component;
use reqwest::Method;
use roxmltree::{Descendants, Document};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Calendar {
    pub(crate) client: Arc<DAVClient>,
    pub(crate) url: Arc<String>,
}

#[derive(Debug, Clone)]
pub struct Etag(String);

impl Calendar {
    /// Saves a new event in the calendar.
    pub async fn save_event(&self, event: icalendar::Event) -> CalDAVResult<EventPointer> {
        let mut calendar = icalendar::Calendar::new();
        calendar.push(event);
        let uid = Uuid::new_v4().to_string();
        let req = self
            .client
            .request(Method::PUT, format!("{}/{}.ics", self.url, &uid))
            .await?
            .header("If-None-Match", "Match")
            .header("Content-Type", "text/calendar")
            .header("Content-Length", "xxxx")
            .body(calendar.to_string());
        let res = req.send().await?;
        if res.status().as_u16() != 201 && res.status() != 200 && res.status().as_u16() != 207 {
            return Err(CalDAVError::OtherError);
        }
        Ok(EventPointer {
            data: AtomicRefCell::new(EventPointerData::CreatedEventResponse { uid }),
            url: self.url.clone(),
            client: self.client.clone(),
        })
    }

    /// Searches for all the events in a specific time period.
    pub async fn date_search(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> CalDAVResult<Vec<EventPointer>> {
        let start = start.format(DATETIME_FORMAT).to_string();
        let end = end.format(DATETIME_FORMAT).to_string();
        let body_string = xml! {
            <?xml version="1.0" encoding="utf-8" ?>
            <C:calendar-query xmlns:D="DAV:"
                          xmlns:C="urn:ietf:params:xml:ns:caldav">
              <D:prop>
                <D:getetag/>
                <C:calendar-data/>
              </D:prop>
              <C:filter>
                <C:comp-filter name="VCALENDAR">
                  <C:comp-filter name="VEVENT">
                    <C:time-range start={start}
                                  end={end}/>
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
            .body(body_string)
            .header("Content-Type", "application/xml; charset=\"utf-8\"")
            .header("Depth", "1")
            .send()
            .await?;
        let text = res.text().await.unwrap();
        let tree = Document::parse(&text).unwrap();
        let res = get_calendar_data(tree.descendants());
        Ok(res
            .map(|item| item.events)
            .flatten()
            .map(|event| EventPointer {
                data: AtomicRefCell::new(EventPointerData::FetchedEvent(event)),
                url: self.url.clone(),
                client: self.client.clone(),
            })
            .collect())
    }
}

pub(crate) fn get_calendar_data<'a>(
    tree: Descendants<'a, 'a>,
) -> impl Iterator<Item = IcalCalendar> + 'a {
    tree.filter(|node| node.tag_name().name() == "calendar-data")
        // this is a bit hard to read (at least it is with my (@teymour-aldridge) editor colour
        // scheme) – it just finds all the calendar events and tries to parse them (if it can't
        // parse an event it just ignores it)
        .filter_map(|node| {
            node.first_child()
                .map(|node| {
                    node.text().map(|text| {
                        ical::IcalParser::new(text.as_bytes())
                            .next()
                            .map(|x| x.map(Some).unwrap_or(None))
                    })
                })
                .flatten()
                .flatten()
                .flatten()
        })
}
