use crate::sync;
use axum::{extract::Path, response::IntoResponse, routing::post, Router};
use common::{Request, RequestId};
use std::net::SocketAddr;
use tokio::sync::mpsc;
use uuid::Uuid;

pub async fn run(tx: mpsc::Sender<sync::SyncMessage>) {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);

    let app = Router::new().route(
        "/*path",
        post(move |body, path| proxy(body, path, tx.clone())),
    );

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}

async fn proxy(
    body: String,
    Path(path): Path<String>,
    sync_tx: mpsc::Sender<sync::SyncMessage>,
) -> impl IntoResponse{
    let request_id = RequestId(Uuid::new_v4().to_string());
    let request = Request {
        request_id,
        path,
        body,
    };
    let response = sync::kafka_get(request, sync_tx).await;
    println!("status {}", response.status);
    (response.status, response.body.unwrap_or_else(||"".to_string()))
}
