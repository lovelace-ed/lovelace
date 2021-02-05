//! Calendaring.

/// Connects calendars to the application.
pub mod connect;

/// Schedules events. Currently we're just recomputing the entire schedule every time something
/// changes. If this is too expensive then we may need to look at doing this incrementally.
pub mod scheduler;

#[cfg(test)]
mod test_calendar;
