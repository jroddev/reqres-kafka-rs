use std::net::SocketAddr;

use axum::{response::IntoResponse, Router, routing::post};


#[tokio::main]
async fn main() {
    let port = 8080;
    println!("Listening on port {port}");
    let app = Router::new()
        .route("/reverse", post(reverse))
        .route("/upper", post(upper));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}


async fn reverse(payload: String) -> impl IntoResponse {
    payload.chars().rev().collect::<String>()
}

async fn upper(payload: String) -> impl IntoResponse {
    payload.to_uppercase()
}
