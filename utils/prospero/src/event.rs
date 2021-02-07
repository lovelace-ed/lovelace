use std::{borrow::Cow, fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

/// The format string passed to chrono to format datetimes.
///
/// **Only use this to format UTC datetimes!!!**
pub(crate) const DATETIME_FORMAT: &str = "%Y%m%dT%H%M%SZ";

/// Construct a new calendar event. Currently this is very rudimentary. In the future we plan to add
/// handling necessary for the complexity of the human experience.
pub struct NewEvent {
    pub(crate) uid: Cow<'static, str>,
    pub(crate) dt_start: DateTime<Utc>,
    pub(crate) dt_end: DateTime<Utc>,
    pub(crate) summary: Cow<'static, str>,
}

impl Display for NewEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BEGIN:VCALENDAR")?;
        f.write_str("\nBEGIN:VEVENT")?;
        f.write_str("\nUID:")?;
        f.write_str(&self.uid)?;
        f.write_str("\nDTSTART:")?;
        println!(
            "DATE: {}",
            &self.dt_start.format(DATETIME_FORMAT).to_string()
        );
        f.write_str(&self.dt_start.format(DATETIME_FORMAT).to_string())?;
        f.write_str("\nDTEND:")?;
        f.write_str(&self.dt_end.format(DATETIME_FORMAT).to_string())?;
        f.write_str("\nSUMMARY:")?;
        f.write_str(&self.summary)?;
        f.write_str("\nEND:VEVENT")?;
        f.write_str("\nEND:VCALENDAR")
    }
}

impl NewEvent {
    pub fn new<S1, S2>(uid: S1, dt_start: DateTime<Utc>, dt_end: DateTime<Utc>, summary: S2) -> Self
    where
        S1: Into<Cow<'static, str>>,
        S2: Into<Cow<'static, str>>,
    {
        Self {
            uid: uid.into(),
            dt_start,
            dt_end,
            summary: summary.into(),
        }
    }
}

/// An ICal event. Because of time constraints this is not an exhaustive list of events, and is not
/// spec-compliant (it is valid to construct iCal events without these fields but this struct will
/// fail to parse).
#[derive(Debug, Clone)]
pub struct Event {
    pub uid: Option<String>,
    pub dt_start: Option<DateTime<Utc>>,
    pub dt_end: Option<DateTime<Utc>>,
    pub summary: Option<String>,
}

#[derive(Error, Debug)]
pub enum ParseCalendarEventError {
    #[error("couldn't parse")]
    CouldntParse,
}

/// A list of properties from an iCal event.
struct Properties {
    props: Vec<Property>,
}

impl Properties {
    /// Parses the properties, removing characters from the string as it does so.
    pub fn parse(string: &mut String) -> Result<Self, ParseCalendarEventError> {
        let mut props = vec![];
        while !string.is_empty() {
            props.push(Property::parse(string)?);
        }
        Ok(Self { props })
    }
}

struct Property {
    k: String,
    v: String,
}

impl Property {
    pub fn parse(string: &mut String) -> Result<Self, ParseCalendarEventError> {
        *string = string.trim().to_string();
        let mut subset = String::new();
        while string.get(0..1) != Some("\n") {
            if string.is_empty() {
                break;
            }
            // don't look at me like that – it panics otherwise
            // yes, this could be improved
            if string.len() == 1 {
                subset.push_str(&string);
                *string = "".to_string();
                break;
            } else {
                subset.push(string.remove(0));
            }
        }
        let mut split = subset.split(':');
        let k = match split.next() {
            Some(t) => t,
            None => return Err(ParseCalendarEventError::CouldntParse),
        }
        .to_string();
        let v = match split.next() {
            Some(t) => t,
            None => return Err(ParseCalendarEventError::CouldntParse),
        }
        .to_string();
        Ok(Self { k, v })
    }
}

fn parse_date(date: &str) -> Result<DateTime<Utc>, ParseCalendarEventError> {
    let naive_date = NaiveDateTime::parse_from_str(date, DATETIME_FORMAT)
        .map_err(|_| ParseCalendarEventError::CouldntParse)?;
    Ok(Utc.from_utc_datetime(&naive_date))
}

impl FromStr for Event {
    type Err = ParseCalendarEventError;

    /// Parses the item from a string. Currently assumes that all dates are in GMT0, and throws a fit
    /// otherwise (todo: fix this)
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut string = s.to_string();
        let mut dt_start = None;
        let mut dt_end = None;
        let mut summary = None;
        let mut uid = None;
        for prop in Properties::parse(&mut string)?.props.into_iter() {
            match prop.k.as_str() {
                "DTSTART" => {
                    dt_start = Some(prop.v);
                }
                "DTEND" => {
                    dt_end = Some(prop.v);
                }
                "SUMMARY" => {
                    summary = Some(prop.v);
                }
                "UID" => uid = Some(prop.v),
                _ => {}
            }
        }
        let uid = match uid {
            Some(t) => Some(t),
            None => None,
        };
        let dt_start = match dt_start.map(|date| parse_date(&date)) {
            Some(t) => Some(t?),
            None => None,
        };
        let dt_end = match dt_end.map(|date| parse_date(&date)) {
            Some(t) => Some(t?),
            None => None,
        };
        Ok(Self {
            uid,
            dt_start,
            dt_end,
            summary,
        })
    }
}

#[cfg(test)]
pub mod test_parse_ical {
    use std::str::FromStr;

    use super::Event;

    const ICAL_EVENTS: &[(&str, &dyn Fn(Event) -> bool)] = &[(
        "BEGIN:VCALENDAR
        \nVERSION:2.0
        \nPRODID:-//hacksw/handcal//NONSGML v1.0//EN
        \nBEGIN:VEVENT
        \nDTSTART:19970714T170000Z
        \nDTEND:19970715T035959Z
        \nSUMMARY:Bastille Day Party
        \nEND:VEVENT
        \nEND:VCALENDAR",
        &|event: Event| event.summary == Some("Bastille Day Party".to_string()),
    )];
    #[test]
    fn test() {
        for event in ICAL_EVENTS {
            assert!(event.1(Event::from_str(event.0).expect("couldn't parse")));
        }
    }
}
