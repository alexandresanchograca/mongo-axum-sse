use axum::{
    response::sse::{Event, Sse},
    routing::get,
    Router,
};

use axum_extra::TypedHeader;
use futures::stream::{Stream, TryStreamExt};
use mongodb::{options::ClientOptions, Client};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, path::PathBuf, time::Duration};
use tokio::sync::broadcast::Sender;
use tokio_stream::StreamExt as _;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct UserModel {
    email: String,
    password: String,
}

async fn send_users_col(users_col: &mongodb::Collection<UserModel>, tx: &Sender<String>) {
    let docs = users_col
        .find(None, None)
        .await
        .unwrap()
        .try_collect::<Vec<UserModel>>()
        .await
        .unwrap();

    println!("Sending user col: {:?}", docs);
    let user_json = serde_json::to_string(&docs).unwrap_or_default();
    let _ = tx.send(user_json);
}

async fn listen_users_col(users_col: mongodb::Collection<UserModel>, tx: Sender<String>) {
    println!("Start users collection listener...");

    let mut change_stream = users_col.watch(None, None).await.unwrap();

    while let Some(users_change) = change_stream.next().await {
        match users_change {
            Ok(_) => send_users_col(&users_col, &tx).await,
            Err(e) => println!("Error: {:?}", e),
        }
    }
}

#[tokio::main]
async fn main() {
    let mongo_db_uri = "mongodb://localhost:27017";
    let db_options = ClientOptions::parse(mongo_db_uri).await.unwrap();
    let client = Client::with_options(db_options).unwrap();
    let my_notes_db = client.database("my-notes");

    let (tx, rx) = tokio::sync::broadcast::channel(32);

    let users_col: mongodb::Collection<UserModel> = my_notes_db.collection("users");
    let users_col_listener = tokio::spawn(listen_users_col(users_col.clone(), tx.clone()));

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // build our application
    let app = app(users_col, tx);

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

fn app(users_col: mongodb::Collection<UserModel>, tx: Sender<String>) -> Router {
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("client_example");
    let static_files_service = ServeDir::new(assets_dir).append_index_html_on_directories(true);
    // build our application with a route
    Router::new()
        .fallback_service(static_files_service)
        .route(
            "/sse",
            get(move |headers| sse_handler(headers, users_col.clone(), tx.clone())),
        )
        .layer(TraceLayer::new_for_http())
}

async fn sse_handler(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
    users_col: mongodb::Collection<UserModel>,
    tx: Sender<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());

    let rx = tx.subscribe();
    let stream = tokio_stream::wrappers::BroadcastStream::new(rx).map(|msg| match msg {
        Ok(payload) => Ok::<Event, Infallible>(Event::default().data(payload)),
        Err(_) => Ok::<Event, Infallible>(Event::default().data("Error receiving message")),
    });

    send_users_col(&users_col, &tx).await;

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
