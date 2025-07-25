use crate::store::ItemStore;
use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

#[debug_handler]
pub async fn delete_item(State(pool): State<ItemStore>, Path(id): Path<i64>) -> impl IntoResponse {
    match pool.delete(id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => {
            tracing::error!(err = %err, "failed to update item");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update item").into_response()
        }
    }
}
