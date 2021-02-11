/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

//! Utilities for sending emails.
//!
//! This acts as an abstraction over a number of ways of sending an email including SMTP and a
//! number of APIs provided by "email as a service" companies.

use serde_json::json;
use thiserror::Error as ThisError;

#[derive(Default, Builder, Debug, Clone)]
pub struct Recipient {
    email: String,
    name: String,
}

#[derive(Default, Builder, Debug, Clone)]
pub struct Recipients {
    recipients: Vec<Recipient>,
}

#[derive(Default, Builder, Debug, Clone)]
pub struct Email {
    recipients: Recipients,
    subject: String,
    plaintext: Option<String>,
    html_text: Option<String>,
    /// A tuple of two strings in the form (Name, Email)
    from: (String, String),
    /// A tuple of two strings in the form (Name, Email)
    reply_to: (String, String),
}

#[derive(ThisError, Debug)]
pub enum EmailSendError {
    #[error("network error")]
    NetworkError,
}

#[rocket::async_trait]
pub trait SendMail {
    /// Sends an email
    async fn send(&self, email: &Email) -> Result<(), EmailSendError>;
}

#[derive(Debug, Default, Clone)]
pub struct SendgridMailSender {}

#[rocket::async_trait]
impl SendMail for SendgridMailSender {
    async fn send(&self, email: &Email) -> Result<(), EmailSendError> {
        let content = {
            let mut result = vec![];
            if let Some(text) = &email.plaintext {
                result.push(json!({
                    "type": "text/plain",
                    "value": text
                }))
            }
            if let Some(text) = &email.html_text {
                result.push(json!({
                    "type": "text/html",
                    "value": text
                }))
            }
            result
        };
        let res = json! ({
                "personalizations": {
                    "to": email
                    .recipients
                    .recipients
                    .iter()
                    .map(|recipient| json! ({"email": recipient.email, "name": recipient.name}))
                    .collect::<Vec<_>>()
                },
                "content": content,
                "from": {
                    "email": email.from.1,
                    "name": email.from.0
                },
                "reply_to": {
                    "email": email.reply_to.1,
                    "name": email.reply_to.0
                }
            }
        );
        match reqwest::Client::new()
            .post(&format!(
                "{}/v3/mail/send",
                std::env::var("SENDGRID_API_SERVER")
                    .unwrap_or_else(|_| "https://api.sendgrid.com".to_string())
            ))
            .header(
                "Authorization",
                &format!(
                    "Bearer: {}",
                    std::env::var("SENDGRID_API_KEY").expect("no sendgrid api key provided")
                ),
            )
            .body(res.to_string())
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("{:#?}", e);
                Err(EmailSendError::NetworkError)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EmailBuilder, RecipientBuilder, RecipientsBuilder};
    use wiremock::{
        matchers::{method, path_regex},
        ResponseTemplate,
    };
    use wiremock::{Mock, MockServer};

    use super::{SendMail, SendgridMailSender};
    /// This test runs in a `tokio` runtime because it "mocks" HTTP requests (i.e. it catches HTTP
    /// requests, so that they are not actually dispatched to the internet.)
    #[tokio::test]
    async fn test_sendgrid_api_sends_correctly() {
        let mock_server = MockServer::start().await;
        std::env::set_var("SENDGRID_API_KEY", "SomeRandomAPIKey");
        std::env::set_var("SENDGRID_API_SERVER", mock_server.uri());
        Mock::given(method("post"))
            .and(path_regex("/v3/mail/send"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;
        let mail_sender = SendgridMailSender::default();
        let result = mail_sender
            .send(
                &EmailBuilder::default()
                    .subject("Some dummy subject".to_string())
                    .plaintext(Some("Hello World!".to_string()))
                    .html_text(Some("<p>Hello World!</p>".to_string()))
                    .recipients(
                        RecipientsBuilder::default()
                            .recipients(vec![RecipientBuilder::default()
                                .email("someone@example.com".to_string())
                                .name("Someone".to_string())
                                .build()
                                .unwrap()])
                            .build()
                            .unwrap(),
                    )
                    .from((
                        "Some dummy sender".to_string(),
                        "dummy_sender@example.com".to_string(),
                    ))
                    .reply_to((
                        "Some dummy sender".to_string(),
                        "dummy_sender@example.com".to_string(),
                    ))
                    .build()
                    .unwrap(),
            )
            .await;
        assert!(result.is_ok());
    }
}
