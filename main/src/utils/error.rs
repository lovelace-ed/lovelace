use malvolio::prelude::*;
use portia::{levels::Level, render::Render};
use rocket::http::Status;
use thiserror::Error as ThisError;

use super::{default_head, json_response::ApiResponse};

#[derive(ThisError, Debug, PartialEq, Clone, Eq)]
pub enum LovelaceError {
    #[error("permission error")]
    PermissionError,
    #[error("database error")]
    DatabaseError,
    #[error("date parsing error")]
    ParseDateError,
    #[error("other error")]
    #[allow(dead_code)]
    OtherError,
}

impl From<diesel::result::Error> for LovelaceError {
    fn from(_: diesel::result::Error) -> Self {
        Self::DatabaseError
    }
}

pub type LovelaceResult<T> = Result<T, LovelaceError>;

impl<T> From<LovelaceError> for ApiResponse<T> {
    fn from(e: LovelaceError) -> Self {
        ApiResponse::new_err(match e {
            LovelaceError::PermissionError => "Permission error",
            LovelaceError::DatabaseError => "Database error",
            LovelaceError::OtherError => "Other error",
            LovelaceError::ParseDateError => "Could not parse one of the dates you supplied.",
        })
    }
}

impl Render<Div> for LovelaceError {
    fn render(self) -> Div {
        match self {
            LovelaceError::PermissionError => Level::new()
                .child(H1::new("Permission error"))
                .child(P::with_text("You don't have permission to do this.")),
            LovelaceError::DatabaseError => {
                Level::new()
                    .child(H1::new("Database error"))
                    .child(P::with_text(
                        "Something's up on our end. This error is a catch-all, and we've logged
                        the fact that this happened and we'll be working out to fix it. This probably
                        doesn't mean that we've made a programming error – instead, there was probably
                        some other reason why we couldn't do this and we just need to update this
                        message to be more informative as to why it happened.",
                    ))
            }
            LovelaceError::OtherError => {Level::new().child(H1::new("Other error"))}
            LovelaceError::ParseDateError => Level::new().child(H1::new("Could not parse one of the dates you supplied."))
        }
        .into_div()
    }
}

impl Render<Html> for LovelaceError {
    fn render(self) -> Html {
        Html::new()
            .status(match self {
                LovelaceError::PermissionError => Status::Forbidden,
                LovelaceError::DatabaseError | LovelaceError::OtherError => {
                    Status::InternalServerError
                }
                LovelaceError::ParseDateError => Status::BadRequest,
            })
            .head(default_head(match self {
                LovelaceError::PermissionError => "Invalid permissions",
                LovelaceError::DatabaseError => "Database error",
                LovelaceError::OtherError => "Unknown error",
                LovelaceError::ParseDateError => "Couldn't parse a provided date",
            }))
            .body(Body::new().child(Render::<Div>::render(self)))
    }
}
