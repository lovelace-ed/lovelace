//! A set of utilities which contain a number of useful error messages.
//!
//! Error messages should be understandable by the person who has to read them (who probably hasn't
//! read through the codebase or has deep familiarity with HTTP)!
use malvolio::prelude::Html;

/// Useful where an error needs to be returned.
pub fn undiagnosed_server_error() -> Html {
    todo!()
}

/// An error with the database.
pub fn database_error() -> Html {
    todo!()
}
