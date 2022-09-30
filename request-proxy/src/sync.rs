use std::sync::{Mutex, Arc};
use std::sync::mpsc::Receiver;
use std::collections::HashMap;
use std::task::Waker;
use uuid::Uuid;
use crate::common::{Request, Response, RequestId};
use crate::kafka;

pub struct FutureHandleData {
    pub data: Option<Response>,
    pub waker: Option<Waker>
}
pub type FutureHandle = Arc<Mutex<FutureHandleData>>;

pub enum Message {
    Request(FutureHandle, Request),
    Response(Response)
}

pub async fn run(sync_rx: Receiver<Message>, kafka_producer: kafka::KafkaProducer, kafka_topic: &str) {
    let mut pending = HashMap::<RequestId, FutureHandle>::new();

    loop {
        let msg = sync_rx.recv();
        match msg {
            Err(e) => eprintln!("Error receiving message in sync thread {e}"),
            Ok(message) => {
                match message {
                    Message::Request(handle, request) => {
                        let request_id = RequestId(Uuid::new_v4().to_string());
                        pending.insert(RequestId(request_id.0.clone()), handle.clone());
                        kafka_producer.produce(&kafka_topic, request_id, request).await;
                    },
                    Message::Response(response) => {
                        let request_id = response.request_id.clone();
                        match pending.get(&request_id) {
                            Some(future) => {
                                match future.lock() {
                                    Ok(mut future) => {
                                       future.data = Some(response);
                                       future.waker.as_ref().unwrap().wake_by_ref();
                                    },
                                    Err(_) => eprintln!("unable to lock Mutex on Future with id {request_id:?}"),
                                }
                            },
                            None => eprintln!("Sync: Response received for id: {request_id:?} but did not existing in pending map"),
                        }
                    },
                }
            },
        }
    }
}
