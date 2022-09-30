use axum::Extension;
use axum::{response::IntoResponse, routing::post, Router};
use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker, Context};
use std::thread;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

mod kafka;

type PendingFutures = Arc<Mutex<HashMap<Uuid, Arc<Mutex<KafkaFutureState>>>>>;

// #[tokio::main]
fn main() {
    let kafka_broker = "localhost:9092";
    let kafka_request_topic = "request";
    let kafka_response_topic = "response";

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.spawn(async move {
        println!("Hello world");
        sleep(Duration::from_secs(10)).await;
        let kafka_producer = kafka::KafkaProducer::new(kafka_broker);
        kafka_producer.produce(
            kafka_request_topic,
            kafka::Request {
                request_id: Uuid::new_v4().to_string(),
                data: String::from("Hello there"),
            },
        ).await;
    });

    runtime.spawn(async move {
        println!("kafka consumer");
        let kafka_consumer = kafka::KafkaConsumer::new("response_listener", kafka_broker, &[kafka_request_topic]);
        loop {
            match kafka_consumer.consume_one().await {
                Ok(message) => {
                    println!("Received message from Kafka: {message:?}");
                }
                Err(e) => eprintln!("KafkaConsumer.KafkaError {e:?}"),
            }
        }
    });

    let pending_futures = Arc::new(Mutex::new(HashMap::new()));

    let shared_copy = Arc::clone(&pending_futures);
    thread::spawn(move || timer_thread(shared_copy));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    let app = Router::new()
        .route("/*path", post(proxy))
        .layer(Extension(Arc::clone(&pending_futures)));

    runtime.block_on(async {
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });
}

fn timer_thread(pending_futures: PendingFutures) {
    println!("start timer thread");
    loop {
        std::thread::sleep(Duration::from_millis(10000));
        println!("timer elapsed");
        let mut pending = pending_futures.lock().unwrap();
        for p in pending.iter_mut() {
            let mut state = p.1.lock().unwrap();
            state.result = Some("working!".to_string());
            if let Some(waker) = &state.waker {
                waker.wake_by_ref();
            }
        }
    }
}

async fn proxy(data: String, state: Extension<PendingFutures>) -> impl IntoResponse {
    println!("request received. data: {data}");
    sleep(Duration::from_millis(100)).await;
    let request_id = Uuid::new_v4();
    let request_handle = KafkaRequest::new(request_id, data, Arc::clone(&state));
    request_handle.await
}

struct KafkaFutureState {
    pub result: Option<String>,
    pub waker: Option<Waker>,
    pub pending: PendingFutures,
}

struct KafkaRequest {
    request_id: Uuid,
    data: String,
    response: Arc<Mutex<KafkaFutureState>>,
}

impl KafkaRequest {
    fn new(request_id: Uuid, data: String, pending_futures: PendingFutures) -> Self {
        KafkaRequest {
            request_id,
            data,
            response: Arc::new(Mutex::new(KafkaFutureState {
                result: None,
                waker: None,
                pending: pending_futures,
            })),
        }
    }
}

// // https://github.com/tokio-rs/tokio/blob/master/tokio/src/time/sleep.rs line 432-455
impl Future for KafkaRequest {
    type Output = String;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut response = self.response.lock().unwrap();
        match &response.result {
            Some(r) => Poll::Ready(r.to_string()),
            None => {
                if response.waker.is_none() {
                    // register waker with other thread
                    let mut pending = response.pending.lock().unwrap();
                    pending.insert(self.request_id, Arc::clone(&self.response));
                }
                // Always update the waker incase task is moved
                response.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}
