/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

//! A set of utilities which contain a number of useful error messages.
//!
//! Error messages should be understandable by the person who has to read them (who probably hasn't
//! read through the codebase or has deep familiarity with HTTP)!
use malvolio::prelude::*;
use rocket::http::Status;

use super::default_head;

/// Return this where an error that shouldn't happen has happened.
#[allow(unused)]
pub fn undiagnosed_server_error() -> Html {
    Html::new()
        .head(default_head("Undiagnosed error"))
        .body(Body::new().child(H1::new("Something strange is happening.")))
}

/// An error with the database.
pub fn database_error() -> Html {
    Html::new()
        .head(Head::new().child(Title::new("Database error")))
        .body(
            Body::new()
                .child(H1::new("Database error"))
                .child(P::with_text(
                    "We ran into some problems retrieving the relevant data from our database.",
                )),
        )
}

pub fn invalid_date(form: Option<Form>) -> Html {
    Html::new()
        .status(Status::BadRequest)
        .head(default_head("Invalid date".to_string()))
        .body(
            Body::new()
                .child(H1::new("Invalid date"))
                .child(P::with_text(
                    "The date you supplied is in an incorrect format.",
                ))
                .map(|body| {
                    if let Some(form) = form {
                        body.child(form)
                    } else {
                        body
                    }
                }),
        )
}
