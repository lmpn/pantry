use crate::item::Item;
use crate::store::ItemStore;
use askama::Template;
use axum::Form;
use axum::response::{Html, Response};
use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateItemForm {
    id: i64,
    name: String,
    quantity: f64,
    state: crate::item::State,
}

#[debug_handler]
pub async fn update_item(
    State(pool): State<ItemStore>,
    Form(form): Form<UpdateItemForm>,
) -> impl IntoResponse {
    match pool
        .update(Item::new(form.id, form.name, form.quantity, form.state))
        .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => {
            tracing::error!(err = %err, "failed to update item");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update item").into_response()
        }
    }
}

#[debug_handler]
pub async fn get_update_item(
    State(pool): State<ItemStore>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match pool.read(id).await {
        Ok(item) => {
            let template =
                UpdateItemFormTemplate::new(item.id, item.name, item.quantity, item.state);
            HtmlTemplate(template).into_response()
        }
        Err(err) => {
            tracing::error!(err = %err, "failed to get requested item");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get requested item",
            )
                .into_response()
        }
    }
}

#[derive(Template)]
#[template(path = "update_item_form.html")]
struct UpdateItemFormTemplate {
    id: i64,
    name: String,
    quantity: f64,
    original_state: crate::item::State,
}

impl UpdateItemFormTemplate {
    fn new(id: i64, name: String, quantity: f64, original_state: crate::item::State) -> Self {
        Self {
            id,
            name,
            quantity,
            original_state,
        }
    }
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}
