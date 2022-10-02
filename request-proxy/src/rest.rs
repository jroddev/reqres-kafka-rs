use std::{
    net::SocketAddr,
    sync::Arc,
};
use tokio::sync::mpsc;
use crate::sync_2;
use common::{Request, RequestId};
use axum::{extract::Path, response::IntoResponse, routing::post, Router};
use uuid::Uuid;

pub async fn run(tx: mpsc::Sender<sync_2::SyncMessage>) {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
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
    sync_tx: mpsc::Sender<sync_2::SyncMessage>,
) -> impl IntoResponse {
    println!("request received. path [{path}]");
    let request_id = RequestId(Uuid::new_v4().to_string());
    let request = Request { request_id, path, body };
    // let pubsub_future = ReqResFuture::new(request, sync_tx);
    // let pubsub_state = Arc::clone(&pubsub_future.state);
    // pubsub_future.await;
    // let pubsub_response = &pubsub_state.lock().unwrap();
    // pubsub_response
    //     .data
    //     .as_ref()
    //     .expect("Future should not be completed without filling response data")
    //     .body
    //     .to_owned()

    let response = sync_2::kafka_get(request, sync_tx).await;
    response.body.unwrap_or_else(|| "NOTHING".to_string())
}
