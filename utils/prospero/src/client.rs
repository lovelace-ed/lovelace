use std::sync::Arc;

use atomic_refcell::AtomicRefCell;

use digest_auth::AuthContext;
use reqwest::{Client, Method, RequestBuilder};
use uuid::Uuid;

pub(crate) const MKCALENDAR: &[u8] = b"MKCALENDAR";
pub(crate) const REPORT: &[u8] = b"REPORT";

use crate::{
    calendar::Calendar,
    error::{CalDavError, CalDavResult},
};

/// The CalDAV client. This is the entry point to the application, and you will need one of these
/// to use all other methods.
#[derive(Debug, Clone)]
pub struct DavClient {
    auth_scheme: AuthScheme,
    url: String,
    client: Client,
    /// This is not neat but it makes the API nicer.
    auth_header: AtomicRefCell<Option<String>>,
}

impl DavClient {
    pub async fn request<S>(&self, method: Method, url: S) -> CalDavResult<RequestBuilder>
    where
        S: AsRef<str>,
    {
        self.authenticate_cache_http(self.client.request(method, url.as_ref()))
            .await
    }
    async fn authenticate_fresh_http(
        &self,
        request: RequestBuilder,
        username: &str,
        password: &str,
    ) -> CalDavResult<RequestBuilder> {
        let res = self.client.get(&self.url).send().await?;
        let headers = res.headers();
        let wwwauth = headers["www-authenticate"].to_str()?;
        let url = self.url.parse::<http::Uri>()?;
        let context = AuthContext::new(username, password, url.path());
        let mut prompt = digest_auth::parse(wwwauth)?;
        let answer = prompt.respond(&context)?.to_header_string();
        Ok(request.header("Authorization", answer))
    }

    async fn authenticate_cache_http(
        &self,
        request: RequestBuilder,
    ) -> CalDavResult<RequestBuilder> {
        match self.auth_scheme {
            AuthScheme::UsernamePassword(ref username, ref password) => {
                if let Some(ref auth) = *self.auth_header.borrow() {
                    Ok(request.header("Authorization", auth))
                } else {
                    self.authenticate_fresh_http(request, username, password)
                        .await
                }
            }
            AuthScheme::None => Ok(request),
        }
    }
    pub fn new_unauthenticated<S>(s: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            auth_scheme: AuthScheme::None,
            url: s.into(),
            client: Client::new(),
            auth_header: AtomicRefCell::new(None),
        }
    }
    /// Construct a new CalDAV client which uses username/password authentication.
    pub fn new_username_password<S1, S2, S3>(username: S1, password: S2, url: S3) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
    {
        Self {
            auth_scheme: AuthScheme::new_username_password(username.into(), password.into()),
            url: url.into(),
            client: Client::new(),
            auth_header: AtomicRefCell::new(None),
        }
    }
    /// Construct a new CalDAV client which uses OAuth authentication.
    pub fn new_oauth(_url: String, _access_token: String, _refresh_token: String) -> Self {
        todo!()
    }
    /// Returns a list of calendars.
    ///
    /// I think the spec discourages using this.
    pub async fn calendars(&'_ self) -> CalDavResult<Vec<Calendar>> {
        let body_string = xml! {
            <?xml version="1.0" encoding="utf-8" ?>
            <D:principal-match xmlns:D="DAV:">
                <D:self/>
                <D:prop>
                    <C:calendar-home-set
                    xmlns:C="urn:ietf:params:xml:ns:caldav"/>
                </D:prop>
            </D:principal-match>
        }
        .to_string();
        self.client
            .request(Method::from_bytes(REPORT).unwrap(), &self.url)
            .body(body_string)
            .send()
            .await?;
        todo!()
    }
    pub fn calendar(&'_ self) -> Calendar {
        Calendar {
            client: Arc::new(self.clone()),
            url: Arc::new(self.url.to_string()),
        }
    }
    /// Creates a new calendar from the provided `MakeCalendar` struct. If any of the fields on the
    /// `MkCalendar` struct are `None` a uuid will be used in their place.
    pub async fn make_calendar(&'_ self, cal: MakeCalendar) -> CalDavResult<Calendar> {
        let url = format!(
            "{}/{}",
            self.url,
            cal.id.unwrap_or_else(|| Uuid::new_v4().to_string())
        );
        let name = if let Some(name) = cal.name {
            name
        } else {
            Uuid::new_v4().to_string()
        };
        let body_string = xml! {
            <?xml version="1.0" encoding="utf-8" ?>
            <C:mkcalendar xmlns:D="DAV"
                          xmlns:C="url:ietf:params:xml:ns:caldav">
                <D:set>
                    <D:prop>
                        <D:displayname>{name}</D:displayname>
                    </D:prop>
                </D:set>
            </C:mkcalendar>
        }
        .to_string();
        self.client
            .request(Method::from_bytes(MKCALENDAR).unwrap(), &url)
            .body(body_string)
            .send()
            .await
            .map(|_| Calendar {
                client: Arc::new(self.clone()),
                url: Arc::new(self.url.to_string()),
            })
            .map_err(CalDavError::RequestError)
    }
}

#[derive(Derivative, Debug)]
#[derivative(Default(new = "true"))]
pub struct MakeCalendar {
    name: Option<String>,
    id: Option<String>,
}

impl MakeCalendar {
    /// Set the `name` of the calendar to be created.
    ///
    /// Note that setting this field is optional and if you do not a uuid (a random string) will be
    /// used in place.
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
    /// Set the `id` of the calendar to be created.
    ///
    /// Note that setting this field is optional and if you do not a uuid (a random string) will be
    /// used in place.
    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }
}

#[derive(Debug, Clone)]
pub enum AuthScheme {
    UsernamePassword(String, String),
    None,
}

impl AuthScheme {
    pub fn new_username_password(username: String, password: String) -> Self {
        Self::UsernamePassword(username, password)
    }
}
