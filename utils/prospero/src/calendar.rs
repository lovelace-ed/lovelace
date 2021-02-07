use std::str::FromStr;

use crate::{
    client::{DAVClient, REPORT},
    error::CalDAVResult,
    event::{Event, NewEvent, DATETIME_FORMAT},
};
use chrono::{DateTime, Utc};
use reqwest::Method;
use roxmltree::Document;
pub struct Calendar<'a> {
    pub(crate) client: &'a DAVClient,
    pub(crate) url: &'a str,
}

pub struct Etag(String);

impl<'a> Calendar<'a> {
    /// Saves a new event in the calendar.
    pub async fn save_event(&self, event: NewEvent) -> CalDAVResult<Etag> {
        let req = self
            .client
            .request(Method::PUT, format!("{}/{}.ics", self.url, event.uid))
            .await?
            .header("If-None-Match", "Match")
            .header("Content-Type", "text/calendar")
            .header("Content-Length", "xxxx")
            .body(event.to_string());
        let res = req.send().await?;
        Ok(Etag(
            res.headers()
                .get("ETag")
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        ))
    }

    /// Searches for all the events in a specific time period.
    pub async fn date_search(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> CalDAVResult<Vec<Event>> {
        let start = start.format(DATETIME_FORMAT).to_string();
        let end = end.format(DATETIME_FORMAT).to_string();
        let res = self
            .client
            .request(Method::from_bytes(REPORT).unwrap(), self.url)
            .await?
            .body(
                xml! {
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
                .to_string(),
            )
            .header("Content-Type", "application/xml; charset=\"utf-8\"")
            .header("Content-Length", "xxxx")
            .header("Depth", "1")
            .send()
            .await?;
        let text = res.text().await.unwrap();
        let tree = Document::parse(&text).unwrap();
        Ok(tree
            .descendants()
            .filter(|node| node.tag_name().name() == "calendar-data")
            // this is a bit hard to read (at least it is with my (@teymour-aldridge) editor colour
            // scheme) – it just finds all the calendar events and tries to parse them (if it can't
            // parse an event it just ignores it)
            .filter_map(|node| {
                dbg!(node);
                node.first_child()
                    .map(|node| {
                        node.text().map(|text| {
                            Event::from_str(text)
                                .map(Some)
                                .map_err(|_| panic!())
                                .unwrap_or(None)
                        })
                    })
                    .flatten()
                    .flatten()
            })
            .map(|item| {
                dbg!(&item);
                item
            })
            .collect())
    }
}
