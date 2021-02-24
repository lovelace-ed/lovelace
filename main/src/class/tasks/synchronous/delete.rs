use diesel::prelude::*;
use malvolio::prelude::*;
use portia::render::Render;
use rocket_contrib::json::Json;

use crate::{
    class::{get_user_role_in_class, tasks::synchronous::AuthCookie, ClassMemberRole},
    db::Database,
    schema::class_synchronous_task,
    utils::{
        default_head,
        error::{LovelaceError, LovelaceResult},
        json_response::ApiResponse,
    },
};

async fn delete_task(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> LovelaceResult<()> {
    if let Some(ClassMemberRole::Teacher) = get_user_role_in_class(auth.0, class_id, &conn).await {
        conn.run(move |c| {
            diesel::delete(
                class_synchronous_task::table
                    .filter(class_synchronous_task::id.eq(task_id))
                    .filter(class_synchronous_task::class_id.eq(class_id)),
            )
            .execute(c)
        })
        .await
        .map(drop)
        .map_err(|e| {
            error!("{:#?}", e);
            LovelaceError::DatabaseError
        })
    } else {
        Err(LovelaceError::PermissionError)
    }
}

#[get("/<class_id>/task/sync/<task_id>/delete")]
pub async fn html_delete_task(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Html {
    match delete_task(class_id, task_id, auth, conn).await {
        Ok(()) => Html::new()
            .head(default_head("Successfully deleted that task".to_string()))
            .body(Body::new().child(H1::new("Successfully deleted that task."))),
        Err(e) => e.render(),
    }
}

#[get("/<class_id>/task/sync/<task_id>/delete")]
pub async fn api_delete_task(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Json<ApiResponse<()>> {
    Json(match delete_task(class_id, task_id, auth, conn).await {
        Ok(()) => ApiResponse::new_ok(()),
        Err(e) => From::from(e),
    })
}
