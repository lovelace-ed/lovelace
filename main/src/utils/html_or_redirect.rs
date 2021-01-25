/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

use malvolio::prelude::Html;
use rocket::response::Redirect;
use rocket::response::Responder;

#[allow(clippy::large_enum_variant)]
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
