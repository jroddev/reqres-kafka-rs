use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::error::{KafkaError, KafkaResult};
use rdkafka::message::{Header, Headers, Message, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::topic_partition_list::TopicPartitionList;
use std::time::Duration;

use common::{Request, RequestId};

pub struct KafkaProducer {
    producer: FutureProducer,
}

impl KafkaProducer {
    pub fn new(brokers: &str) -> KafkaProducer {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");

        KafkaProducer { producer }
    }

    pub async fn produce(&self, topic: &str, request: Request) {
        let headers = OwnedHeaders::new()
            .insert(Header {
                key: "request_id",
                value: Some(&request.request_id.0),
            })
            .insert(Header {
                key: "path",
                value: Some(&request.path),
            });

        let key = "0";
        let delivery_status = self.producer.send(
            FutureRecord::to(topic)
                .payload(&request.body)
                .key(key)
                .headers(headers),
            Duration::from_secs(0),
        );
        if let Err(e) = delivery_status.await {
            eprintln!("{:?}", e)
        }
    }
}

pub struct KafkaConsumer {
    consumer: LoggingConsumer,
}

impl KafkaConsumer {
    pub fn new(name: &str, brokers: &str, topics: &[&str]) -> KafkaConsumer {
        let context = CustomContext;

        let consumer: LoggingConsumer = ClientConfig::new()
            .set("group.id", name)
            .set("bootstrap.servers", brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "latest")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create_with_context(context)
            .expect("Consumer creation failed");

        consumer
            .subscribe(topics)
            .expect("Can't subscribe to specified topics");

        println!("KafkaConsumer [{name}] listening on {topics:?}");
        KafkaConsumer { consumer }
    }

    pub async fn consume_one(&self) -> Result<Request, KafkaError> {
        let message = self.consumer.recv().await?;
        let response: Option<Request> = {
            let request_id: Option<String> = match message.headers() {
                Some(headers) => headers
                    .iter()
                    .filter(|h| h.value.is_some())
                    .find(|h| h.key == "request_id")
                    .map(|h| h.value.unwrap())
                    .map(|v| String::from_utf8_lossy(v).to_string()),
                None => None,
            };

            let request_path: String = match message.headers() {
                Some(headers) => headers
                    .iter()
                    .filter(|h| h.value.is_some())
                    .find(|h| h.key == "path")
                    .map(|h| h.value.unwrap())
                    .map(|v| String::from_utf8_lossy(v).to_string())
                    .unwrap_or_else(|| String::from("/")),
                None => String::from("/"),
            };

            let payload: String = message
                .payload()
                .map(|p| String::from_utf8_lossy(p).to_string())
                .unwrap_or_else(|| String::from(""));

            request_id.map(|request_id| Request {
                request_id: RequestId(request_id),
                path: request_path,
                body: payload,
            })
        };

        self.consumer
            .commit_message(&message, CommitMode::Async)
            .unwrap();

        match response {
            Some(r) => Ok(r),
            None => {
                panic!("could not parse consumed message into Response object");
            }
        }
    }
}

struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        println!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        println!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        println!("Committing offsets: {:?}", result);
    }
}

// A type alias with your custom consumer can be created for convenience.
type LoggingConsumer = StreamConsumer<CustomContext>;
