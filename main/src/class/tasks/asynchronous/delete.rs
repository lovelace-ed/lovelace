use diesel::prelude::*;
use malvolio::prelude::*;
use rocket_contrib::json::Json;
use thiserror::Error as ThisError;

use crate::{
    auth::AuthCookie,
    db::Database,
    utils::{default_head, error_messages::database_error, json_response::ApiResponse},
};

#[derive(ThisError, Debug)]
pub enum DeleteTaskError {
    #[error("database error")]
    DatabaseError,
}

async fn delete_task(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Result<(), DeleteTaskError> {
    use crate::schema::class_asynchronous_task::dsl as class_asynchronous_task;
    conn.run(move |c| {
        diesel::delete(
            class_asynchronous_task::class_asynchronous_task
                .filter(class_asynchronous_task::id.eq(task_id))
                .filter(class_asynchronous_task::class_id.eq(class_id)),
        )
        .execute(c)
    })
    .await
    .map_err(|e| {
        error!("{:#?}", e);
        DeleteTaskError::DatabaseError
    })
    .map(drop)
}

#[get("/<class_id>/task/async/<task_id>/delete")]
pub async fn html_delete_task(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Html {
    match delete_task(class_id, task_id, auth, conn).await {
        Ok(_) => Html::new()
            .head(default_head("Successfully deleted that task".to_string()))
            .body(Body::new().child(H1::new("Successfully deleted that task."))),
        Err(_) => database_error(),
    }
}

#[get("/<class_id>/task/async/<task_id>/delete")]
pub async fn api_delete_task(
    class_id: i32,
    task_id: i32,
    auth: AuthCookie,
    conn: Database,
) -> Json<ApiResponse<()>> {
    Json(match delete_task(class_id, task_id, auth, conn).await {
        Ok(_) => ApiResponse::new_ok(()),
        Err(_) => ApiResponse::new_err("database error"),
    })
}
