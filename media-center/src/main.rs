use askama::Template;
use axum::response::{Html, IntoResponse};
use axum::routing::post;
use axum::{Router, routing::get};
use sqlx::{FromRow, Pool};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

#[derive(Template, Debug, Clone)]
#[template(path = "file_navigator.html")]
struct FileNavigatorTemplate {
    media_items: Vec<MediaItem>,
}

#[derive(Debug, Clone, sqlx::Type, serde::Serialize, serde::Deserialize)]
pub enum MediaKind {
    Movie,
    Series,
    Music,
    Image,
    Other,
}

#[derive(Debug, Clone, FromRow)]
struct MediaItem {
    id: i64,
    title: String,
    path: String,
    kind: MediaKind,
    conceptual_path: Option<String>,
}

async fn get_media_items(
    pool: &Pool<sqlx::Sqlite>,
    filter: MediaFilter,
) -> Result<Vec<MediaItem>, sqlx::Error> {
    let media: Vec<MediaItem> = sqlx::query_as(
        "SELECT id, title, path, kind, conceptual_path FROM media WHERE conceptual_path = ?",
    )
    .bind(filter.conceptual_path)
    .fetch_all(pool)
    .await?;
    Ok(media)
}

use axum::extract::{Multipart, Query, State};

#[derive(Debug, serde::Deserialize)]
struct MediaFilter {
    conceptual_path: String,
}

async fn index(
    State(pool): State<Pool<sqlx::Sqlite>>,
    Query(filter): Query<MediaFilter>,
) -> impl IntoResponse {
    let media_items = get_media_items(&pool, filter)
        .await
        .unwrap_or_else(|_| vec![]);
    Html(FileNavigatorTemplate { media_items }.render().unwrap())
}

async fn upload_file(
    State(pool): State<Pool<sqlx::Sqlite>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let upload_dir = "uploads"; // Define the directory where files will be uploaded
    if let Err(e) = fs::create_dir_all(upload_dir).await {
        eprintln!("Failed to create upload directory: {}", e);
        return Html(format!(
            "<h1>Error: Failed to create upload directory</h1><p>{}</p>",
            e
        ));
    }

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let file_name = field.file_name().map(|s| s.to_string());
        let content_type = field.content_type().map(|s| s.to_string());
        let data = field.bytes().await.unwrap();

        println!(
            "Field received: Name: {:?}, File Name: {:?}, Content Type: {:?}, Size: {} bytes",
            name,
            file_name,
            content_type,
            data.len()
        );

        if let Some(filename) = file_name {
            let path = format!("{}/{}", upload_dir, filename);
            match tokio::fs::File::create(&path).await {
                Ok(file) => {
                    return handle_file(&pool, content_type, data, &path, file, &filename).await;
                }
                Err(e) => {
                    eprintln!("Failed to create file {}: {}", path, e);
                    return Html(format!(
                        "<h1>Error creating file: {}</h1><p>{}</p>",
                        filename, e
                    ));
                }
            }
        }
    }

    Html("<h1>No file uploaded.</h1>".to_string())
}

async fn handle_file(
    pool: &Pool<sqlx::Sqlite>,
    content_type: Option<String>,
    data: axum::body::Bytes,
    path: &String,
    mut file: fs::File,
    filename: &String,
) -> Html<String> {
    if let Err(e) = file.write_all(&data).await {
        eprintln!("Failed to write file {}: {}", path, e);
        return Html(format!(
            "<h1>Error writing file: {}</h1><p>{}</p>",
            filename, e
        ));
    }
    println!("File saved to: {}", path);
    let inferred_kind = infer_media_kind(&content_type);
    match sqlx::query("INSERT INTO media (title, path, kind, conceptual_path) VALUES (?, ?, ?, ?)")
        .bind(filename)
        .bind(path)
        .bind(inferred_kind)
        .bind("/uploads")
        .execute(pool)
        .await
    {
        Ok(_) => println!("Database entry created for {}", filename),
        Err(e) => {
            eprintln!("Failed to insert media item into DB: {}", e);
            return Html(format!(
                "<h1>Error writing file: {}</h1><p>{}</p>",
                filename, e
            ));
        }
    }
    return Html(format!("<h1>File uploaded successfully: {}</h1>", filename));
}

fn infer_media_kind(content_type: &Option<String>) -> MediaKind {
    if let Some(ct) = content_type {
        if ct.starts_with("video/") {
            MediaKind::Movie
        } else if ct.starts_with("audio/") {
            MediaKind::Music
        } else if ct.starts_with("image/") {
            MediaKind::Image
        } else {
            MediaKind::Other
        }
    } else {
        MediaKind::Other
    }
}

#[tokio::main]
async fn main() {
    // Set up database connection pool
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = Pool::connect(&database_url)
        .await
        .expect("Failed to create pool.");

    // Run migrations
    // This is handled by sqlx-cli before running the application

    // Add the pool to the router state
    let app = Router::new()
        .route("/", get(index))
        .route("/upload", post(upload_file)) // New endpoint for file uploads
        .with_state(pool);

    let listener = TcpListener::bind(&"[::]:3000")
        .await
        .expect("failed to create tcplistener");

    // Run the server with graceful shutdown
    axum::serve(listener, app.into_make_service())
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
        _ = ctrl_c => {}, //
        _ = terminate => {}, //
    }
}
