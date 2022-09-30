use std::{
    net::SocketAddr,
    sync::{mpsc::Sender, Arc},
};

use crate::{sync, common::RequestId};
use crate::{common::Request, future::ReqResFuture};
use axum::{extract::Path, response::IntoResponse, routing::post, Router};
use uuid::Uuid;

pub async fn run(tx: Sender<sync::Message>) {
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
    sync_tx: Sender<sync::Message>,
) -> impl IntoResponse {
    println!("request received. path [{path}] body [{body}]");
    let request_id = RequestId(Uuid::new_v4().to_string());
    let request = Request { request_id, path, body };
    let pubsub_future = ReqResFuture::new(request, sync_tx);
    let pubsub_state = Arc::clone(&pubsub_future.state);
    pubsub_future.await;
    let pubsub_response = &pubsub_state.lock().unwrap();
    let pubsub_response_data = pubsub_response
        .data
        .as_ref()
        .expect("Future should not be completed without filling response data");
    match &pubsub_response_data.body {
        Some(text) => text.to_owned(),
        None => String::from(""),
    }
}
