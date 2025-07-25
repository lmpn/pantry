use crate::store::ItemStore;
use askama::Template;
use axum::extract::Query;
use axum::response::{Html, Response};
use axum::{debug_handler, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

struct StatePresentation<'a> {
    id: &'a str,
    name: &'a str,
    move_description: &'a str,
    css_color: &'a str,
}

const STOCK_PRESENTATION: StatePresentation<'static> = StatePresentation {
    id: "stock",
    name: "Stock",
    move_description: "Move to Stock",
    css_color: "bg-success",
};

const SHOPPING_PRESENTATION: StatePresentation<'static> = StatePresentation {
    id: "shopping",
    name: "Shopping",
    move_description: "Move to Shopping",
    css_color: "bg-warning",
};

const STOCK_TRANSITIONS: &[StatePresentation; 1] = &[SHOPPING_PRESENTATION];
const SHOPPING_TRANSITIONS: &[StatePresentation; 1] = &[STOCK_PRESENTATION];

struct ItemTemplate {
    id: i64,
    name: String,
    quantity: f64,
}

impl ItemTemplate {
    fn new(id: i64, name: String, quantity: f64) -> Self {
        Self { id, name, quantity }
    }
}

#[derive(Template)]
#[template(path = "state_items.html")]
struct StateItemsTemplate {
    state: StatePresentation<'static>,
    items: Vec<ItemTemplate>,
    transitions: &'static [StatePresentation<'static>; 1],
}

impl StateItemsTemplate {
    fn new(state: crate::item::State, items: Vec<ItemTemplate>) -> Self {
        let (state, transitions) = match state {
            crate::item::State::Stock => (STOCK_PRESENTATION, STOCK_TRANSITIONS),
            crate::item::State::Shopping => (SHOPPING_PRESENTATION, SHOPPING_TRANSITIONS),
        };
        Self {
            state,
            items,
            transitions,
        }
    }
}

#[derive(Template)]
#[template(path = "state_items_error.html")]
struct StateItemsErrorTemplate {
    error_message: String,
}

impl StateItemsErrorTemplate {
    fn new(error_message: String) -> Self {
        Self { error_message }
    }
}

#[derive(Deserialize)]
pub struct QueryParameters {
    state: crate::item::State,
}

#[debug_handler]
pub async fn state_items(
    State(pool): State<ItemStore>,
    Query(query): Query<QueryParameters>,
) -> impl IntoResponse {
    let items = match pool.read_many_from_state(query.state).await {
        Ok(items) => items,
        Err(err) => {
            tracing::error!(err = %err, state = %query.state, "failed to read items from state");
            let name = match query.state {
                crate::item::State::Stock => STOCK_PRESENTATION.name,
                crate::item::State::Shopping => SHOPPING_PRESENTATION.name,
            };

            let template = StateItemsErrorTemplate::new(format!("Failed to get items in {name}."));
            return HtmlTemplate(template, StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    };

    let items = items
        .iter()
        .map(|item| ItemTemplate::new(item.id, item.name.clone(), item.quantity))
        .collect();

    let template = StateItemsTemplate::new(query.state, items);
    HtmlTemplate(template, StatusCode::OK).into_response()
}

struct HtmlTemplate<T>(T, StatusCode);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => (self.1, Html(html)).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}
