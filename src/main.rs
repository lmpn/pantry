use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use config::Config;
use configuration::Configuration;
use store::SqliteItemStore;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod configuration;
mod create_item;
mod delete_item;
mod index;
mod item;
mod state_items;
mod store;
mod update_item;

#[tokio::main]
async fn main() {
    let configuration: Configuration = Config::builder()
        // Add in `./Settings.toml`
        .add_source(config::File::with_name("config.toml"))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .and_then(|config| config.try_deserialize())
        .expect("failed to load configuration");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let store = Arc::new(SqliteItemStore::new(&configuration.database.dsn).await);
    let app = Router::new()
        .route("/", get(index::index))
        .route("/item", post(create_item::create_item))
        .route("/item", get(state_items::state_items))
        .route("/item", put(update_item::update_item))
        .route("/item/{id}", delete(delete_item::delete_item))
        .route("/item/edit-form/{id}", get(update_item::get_update_item))
        .layer(TraceLayer::new_for_http())
        .layer(tower_http::timeout::TimeoutLayer::new(
            std::time::Duration::from_secs(10),
        ))
        .with_state(store);

    // Create a `TcpListener` using tokio.
    let listener = TcpListener::bind(format!(
        "{}:{}",
        configuration.server.host, configuration.server.port
    ))
    .await
    .expect("failed to create tcplistener");

    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
