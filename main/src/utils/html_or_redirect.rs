use malvolio::Html;
use rocket::response::Redirect;
use rocket::response::Responder;

pub enum HtmlOrRedirect {
    Html(Html),
    Redirect(Redirect),
}

impl<'r> Responder<'r> for HtmlOrRedirect {
    fn respond_to(self, request: &rocket::Request) -> rocket::response::Result<'r> {
        match self {
            HtmlOrRedirect::Html(h) => h.respond_to(request),
            HtmlOrRedirect::Redirect(r) => r.respond_to(request),
        }
    }
}
