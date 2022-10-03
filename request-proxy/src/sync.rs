use common::{Request, RequestId};
use kafka;
use std::alloc::handle_alloc_error;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::task::Waker;

#[derive(Debug)]
pub struct FutureHandleData {
    pub data: Option<Request>,
    pub waker: Option<Waker>,
}
pub type FutureHandle = Arc<Mutex<FutureHandleData>>;

#[derive(Debug)]
pub enum Message {
    Request(RequestId, FutureHandle, Request),
    Response(Request),
}


// pub async fn trigger(request: Request) -> Request {
//     let (tx, rx) = oneshot::<Request>::channel();
//     kafka_produce(request, tx);
//     rx.await
// }

pub async fn run(
    sync_rx: Receiver<Message>,
    kafka_producer: kafka::KafkaProducer,
    kafka_topic: &str,
) {
    let mut pending = HashMap::<RequestId, FutureHandle>::new();

    loop {
        let msg = sync_rx.recv();
        match msg {
            Err(e) => eprintln!("Error receiving message in sync thread {e}"),
            Ok(message) => {
                match message {
                    Message::Request(request_id, handle, request) => {
                        let already_submitted = pending.contains_key(&request_id);
                        pending.insert(RequestId(request_id.0.clone()), handle.clone());

                        if !already_submitted {
                            kafka_producer
                                .produce(kafka_topic, request)
                                .await;
                            }
                    }
                    Message::Response(response) => {
                        let request_id = response.request_id.clone();
                        match pending.get(&request_id) {
                            Some(future) => {
                                match future.lock() {
                                    Ok(mut future) => {
                                       future.data = Some(response);
                                       match &future.waker {
                                            Some(w) => {
                                                w.wake_by_ref();
                                            },
                                            None => panic!("No waker when response received"),
                                        }
                                    },
                                    Err(_) => eprintln!("unable to lock Mutex on Future with id {request_id:?}"),
                                }
                            },
                            None => eprintln!("Sync: Response received for id: {request_id:?} but did not existing in pending map"),
                        }
                    }
                }
            }
        }
    }
}
