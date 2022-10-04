use common::Request;
use reqwest::{self, Client};

#[tokio::main]
async fn main() {
    let rest_api = "http://rest-api:8080";
    let kafka_broker = "kafka:29092";
    let kafka_request_topic = "request";
    let kafka_response_topic = "response";

    let http_client = Client::new();
    let kafka_producer = kafka::KafkaProducer::new(kafka_broker);
    let kafka_consumer =
        kafka::KafkaConsumer::new("request_listener", kafka_broker, &[kafka_request_topic]);

    loop {
        match kafka_consumer.consume_one().await {
            Ok(request) => {
                let response_body = match http_client
                    .post(format!("{}{}", rest_api, request.path))
                    .body(request.body)
                    .send()
                    .await {
                        Ok(b) => b.text().await.unwrap(),
                        Err(_) => String::from(""),
                    };

                kafka_producer
                    .produce(kafka_response_topic, Request{
                        body: response_body,
                        ..request
                    })
                    .await;
            }
            Err(e) => eprintln!("KafkaConsumer.KafkaError {e:?}"),
        }
    }
}
