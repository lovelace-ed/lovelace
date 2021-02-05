/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

use malvolio::prelude::Html;
use rocket::response::Redirect;
use rocket::response::Responder;

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum HtmlOrRedirect {
    Html(Html),
    Redirect(Redirect),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for HtmlOrRedirect {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        match self {
            HtmlOrRedirect::Html(h) => h.respond_to(request),
            HtmlOrRedirect::Redirect(r) => r.respond_to(request),
        }
    }
}
