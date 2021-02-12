use http::uri::InvalidUri;
use reqwest::{header::ToStrError, Error};

#[derive(Error, Debug)]
pub enum CalDavError {
    #[error("request error")]
    RequestError(Error),
    #[error("other error")]
    OtherError,
}

impl From<digest_auth::Error> for CalDavError {
    fn from(_: digest_auth::Error) -> Self {
        Self::OtherError
    }
}

impl From<InvalidUri> for CalDavError {
    fn from(_: InvalidUri) -> Self {
        CalDavError::OtherError
    }
}

impl From<ToStrError> for CalDavError {
    fn from(_: ToStrError) -> Self {
        CalDavError::OtherError
    }
}

impl From<Error> for CalDavError {
    fn from(e: Error) -> Self {
        Self::RequestError(e)
    }
}

pub type CalDavResult<T> = Result<T, CalDavError>;
