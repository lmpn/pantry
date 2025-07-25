use crate::{item::Item, store::ItemStore};
use axum::{Form, debug_handler, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateItemForm {
    name: String,
    quantity: f64,
    state: crate::item::State,
}

#[debug_handler]
pub async fn create_item(
    State(pool): State<ItemStore>,
    Form(form): Form<CreateItemForm>,
) -> impl IntoResponse {
    match pool
        .create(Item::new(0, form.name, form.quantity, form.state))
        .await
    {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(err) => {
            tracing::error!(err = %err, "failed to create item");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create item").into_response()
        }
    }
}
