/*
This source code file is distributed subject to the terms of the GNU Affero General Public License.
A copy of this license can be found in the `licenses` directory at the root of this project.
*/

use crate::{
    auth::AuthCookie,
    class::get_user_role_in_class,
    db::Database,
    utils::{default_head, error_message},
};

use super::ClassMemberRole;

use malvolio::prelude::*;

#[get("/class/<id>/settings")]
pub async fn get_class_settings(id: usize, auth_cookie: AuthCookie, conn: Database) -> Html {
    if get_user_role_in_class(auth_cookie.0 as i32, id as i32, &conn).await
        == Some(ClassMemberRole::Teacher)
    {
        Html::default()
            .head(default_head("Settings".to_string()))
            .body(
                Body::default().child(H1::new("Settings")).child(
                    Div::new().child(
                        A::default()
                            .attribute(Href::new(format!("/class/{}/delete", id)))
                            .text("Delete this class."),
                    ),
                ),
            )
    } else {
        error_message(
            "Insufficient permissions.".to_string(),
            "You need to be a teacher for this class to see it's settings.".to_string(),
        )
    }
}
