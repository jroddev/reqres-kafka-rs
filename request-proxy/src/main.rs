mod rest;
mod sync;

use tokio::sync::mpsc;

fn main() {
    let kafka_broker = "kafka:29092";
    let kafka_request_topic = "request";
    let kafka_response_topic = "response";

    let (tx, rx) = mpsc::channel::<sync::SyncMessage>(100);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.spawn(async move {
        println!("sync thread");
        let kafka_producer = kafka::KafkaProducer::new(kafka_broker);
        sync::run(rx, kafka_producer, kafka_request_topic).await;
    });

    let kafka_consume_tx = tx.clone();
    runtime.spawn(async move {
        println!("kafka consumer thread");
        let kafka_consumer =
            kafka::KafkaConsumer::new("response_listener", kafka_broker, &[kafka_response_topic]);
        loop {
            match kafka_consumer.consume_one().await {
                Ok(message) => {
                    kafka_consume_tx
                        .send(sync::SyncMessage::Response(message))
                        .await
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
