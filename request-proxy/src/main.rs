use std::sync::mpsc;

use crate::common::Response;

mod common;
mod future;
mod kafka;
mod rest;
mod sync;

fn main() {
    let kafka_broker = "localhost:9092";
    let kafka_request_topic = "request";
    let kafka_response_topic = "response";

    let (tx, rx) = mpsc::channel::<sync::Message>();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.spawn(async move {
        println!("sync thread");
        let kafka_producer = kafka::KafkaProducer::new(kafka_broker);
        sync::run(rx, kafka_producer, &kafka_request_topic).await;
    });

    let kafka_consume_tx = tx.clone();
    runtime.spawn(async move {
        println!("kafka consumer thread");
        let kafka_consumer =
            kafka::KafkaConsumer::new("response_listener", kafka_broker, &[kafka_response_topic]);
        loop {
            match kafka_consumer.consume_one().await {
                Ok(message) => {
                    println!("Received message from Kafka: {message:?}");
                    kafka_consume_tx
                        .send(sync::Message::Response(Response {
                            request_id: message.request_id,
                            data: message.data,
                        }))
                        .unwrap();
                }
                Err(e) => eprintln!("KafkaConsumer.KafkaError {e:?}"),
            }
        }
    });

    runtime.block_on(async {
        println!("http thread");
        rest::run(tx).await;
    });
}
