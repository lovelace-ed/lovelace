use http::uri::InvalidUri;
use reqwest::{header::ToStrError, Error};

#[derive(Error, Debug)]
pub enum CalDAVError {
    #[error("request error")]
    RequestError(Error),
    #[error("other error")]
    OtherError,
}

impl From<digest_auth::Error> for CalDAVError {
    fn from(_: digest_auth::Error) -> Self {
        Self::OtherError
    }
}

impl From<InvalidUri> for CalDAVError {
    fn from(_: InvalidUri) -> Self {
        CalDAVError::OtherError
    }
}

impl From<ToStrError> for CalDAVError {
    fn from(_: ToStrError) -> Self {
        CalDAVError::OtherError
    }
}

impl From<Error> for CalDAVError {
    fn from(e: Error) -> Self {
        Self::RequestError(e)
    }
}

pub type CalDAVResult<T> = Result<T, CalDAVError>;
