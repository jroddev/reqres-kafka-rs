use axum::http::status::StatusCode;
use std::{collections::HashMap, thread, time::Duration};
use tokio::{sync::{mpsc, oneshot}, time::sleep};

use common::{Request, RequestId};

#[derive(Debug)]
pub enum SyncMessage {
    Request(Request, oneshot::Sender<Response>),
    Response(Request),
    Timeout(RequestId),
}

#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub body: Option<String>,
}

pub async fn kafka_get(request: Request, sync_tx: mpsc::Sender<SyncMessage>) -> Response {
    let (tx, rx) = oneshot::channel::<Response>();
    let sleep_req_id = request.request_id.clone();

    if let Err(e) = sync_tx.send(SyncMessage::Request(request, tx)).await {
        eprintln!("send error: {e}");
    }

    thread::spawn(|| async move {
        sleep(Duration::from_secs(10)).await;
        if let Err(e) = sync_tx.send(SyncMessage::Timeout(sleep_req_id)).await {
            eprintln!("sleep error {e}");
        }
    });

    match rx.await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("receive error: {e}");
            Response {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                body: None,
            }
        }
    }
}

pub async fn run(
    mut sync_rx: mpsc::Receiver<SyncMessage>,
    kafka_producer: kafka::KafkaProducer,
    kafka_topic: &str,
) {
    let mut pending = HashMap::<RequestId, oneshot::Sender<Response>>::new();

    loop {
        match sync_rx.recv().await {
            Some(msg) => match msg {
                SyncMessage::Request(request, reply_tx) => {
                    pending.insert(request.request_id.clone(), reply_tx);
                    kafka_producer.produce(kafka_topic, request).await;
                }
                SyncMessage::Response(res) => {
                    let request_id = res.request_id;
                    if let Some(request) = pending.remove(&request_id) {
                        let ok = Response {
                            status: StatusCode::OK,
                            body: Some(res.body),
                        };
                        if let Err(e) = request.send(ok) {
                            eprintln!("Failed to ok request {request_id:?}. error: {e:?}");
                        }
                    }
                }
                SyncMessage::Timeout(request_id) => {
                    if let Some(request) = pending.remove(&request_id) {
                        let timeout = Response {
                            status: StatusCode::REQUEST_TIMEOUT,
                            body: None,
                        };
                        if let Err(e) = request.send(timeout) {
                            eprintln!("Failed to timeout request {request_id:?}. error: {e:?}");
                        }
                    }
                }
            },
            None => {
                eprintln!("Sync thread received none on channel. Likely closed");
                return;
            }
        };
    }
}
