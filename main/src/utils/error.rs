use malvolio::prelude::*;
use portia::{levels::Level, render::Render};
use thiserror::Error as ThisError;

use super::{default_head, json_response::ApiResponse};

#[derive(ThisError, Debug)]
pub enum LovelaceError {
    #[error("permission error")]
    PermissionError,
    #[error("database error")]
    DatabaseError,
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
        }
        .into_div()
    }
}

impl Render<Html> for LovelaceError {
    fn render(self) -> Html {
        Html::new()
            .status(match self {
                LovelaceError::PermissionError => 403,
                LovelaceError::DatabaseError => 500,
            })
            .head(default_head(match self {
                LovelaceError::PermissionError => "Invalid permissions",
                LovelaceError::DatabaseError => "Database error",
            }))
            .body(Body::new().child(Render::<Div>::render(self)))
    }
}
