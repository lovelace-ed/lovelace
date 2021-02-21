use malvolio::prelude::*;
use rocket_contrib::json::Json;

use crate::{
    auth::AuthCookie,
    db::Database,
    utils::{default_head, error::LovelaceError, json_response::ApiResponse},
};

use super::{get_user_role_in_class, ClassMemberRole};

#[get("/class/<id>")]
pub async fn html_view_class_overview(id: usize, auth_cookie: AuthCookie, conn: Database) -> Html {
    match get_user_role_in_class(auth_cookie.0 as i32, id as i32, &conn).await {
        Some(role) => match role {
            ClassMemberRole::Student => {
                let class = crate::models::Class::with_id(id as i32, &conn)
                    .await
                    .unwrap();
                Html::default()
                    .head(default_head(class.name.to_string()))
                    .body(
                        Body::default()
                            .child(H1::new(format!("Class: {}", class.name)))
                            .child(P::with_text(class.description)),
                    )
            }
            ClassMemberRole::Teacher => {
                let class = crate::models::Class::with_id(id as i32, &conn)
                    .await
                    .unwrap();
                Html::default().head(default_head(class.name.clone())).body(
                    Body::default()
                        .child(H1::new(format!("Class: {}", class.name)))
                        .child(H3::new(format!(
                            "Invite people to join with the code: {}",
                            class.code
                        )))
                        .child(
                            P::with_text(class.description).child(
                                A::default()
                                    .attribute(Href::new(format!("/class/{}/settings", class.id)))
                                    .text("Settings".to_string()),
                            ),
                        ),
                )
            }
        },
        None => Html::default()
            .head(default_head("Invalid permission".to_string()))
            .body(
                Body::default()
                    .child(H1::new("You don't have permission to view this class."))
                    .child(P::with_text(
                        "You might need to ask your teacher for an invite code.",
                    )),
            ),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudentClass {
    name: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClassOverview {
    Teacher(crate::models::Class),
    Student(StudentClass),
}

#[get("/class/<id>")]
pub async fn api_view_class_overview(
    id: usize,
    auth_cookie: AuthCookie,
    conn: Database,
) -> Json<ApiResponse<ClassOverview>> {
    Json(
        match get_user_role_in_class(auth_cookie.0 as i32, id as i32, &conn).await {
            Some(role) => match role {
                ClassMemberRole::Student => {
                    let class = crate::models::Class::with_id(id as i32, &conn)
                        .await
                        .unwrap();
                    ApiResponse::new_ok(ClassOverview::Student(StudentClass {
                        name: class.name,
                        description: class.description,
                    }))
                }
                ClassMemberRole::Teacher => {
                    let class = crate::models::Class::with_id(id as i32, &conn)
                        .await
                        .unwrap();
                    ApiResponse::new_ok(ClassOverview::Teacher(class))
                }
            },
            None => LovelaceError::PermissionError.into(),
        },
    )
}
