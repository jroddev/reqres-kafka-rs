use std::time::Duration;
use uuid::Uuid;

use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::topic_partition_list::TopicPartitionList;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::error::KafkaResult;
use rdkafka::message::{OwnedHeaders, Message, Headers, HeadersIter, Header};
use rdkafka::producer::{FutureProducer, FutureRecord};
// use rdkafka::util::get_rdkafka_version;




// A context can be used to change the behavior of producers and consumers by adding callbacks
// that will be executed by librdkafka.
// This particular context sets up custom callbacks to log rebalancing events.
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


pub struct KafkaConnection {
    pub brokers: String,
    pub timeout_ms: u32,
}

impl KafkaConnection {

    pub async fn produce(&self, topic: &str, request_id: &str, data: &str) {
        let producer: &FutureProducer = &ClientConfig::new()
            .set("bootstrap.servers", &self.brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");


        let headers = OwnedHeaders::new().insert(Header{
            key: "request_id",
            value: Some(request_id)
        });
        let key = "0";
        let delivery_status = producer.send( FutureRecord::to(topic)
                .payload(data)
                .key(key)
                .headers(headers), Duration::from_secs(0));
        println!("kafka sent");
        match delivery_status.await {
            Err(e) => eprintln!("{:?}", e),
            Ok(v) => println!("kafka sent confirmed. {:?}", v)
        }
    }

    pub async fn consume(&self, topics: &[&str]) {
        let context = CustomContext;
        let group_id = "consumer".to_string(); //Uuid::new_v4().to_string();

        let consumer: LoggingConsumer = ClientConfig::new()
            .set("group.id", &group_id)
            .set("bootstrap.servers", &self.brokers)
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

        println!("subscribed to {topics:?}");

        loop {
            match consumer.recv().await {
                Err(e) => eprintln!("Kafka error: {}", e),
                Ok(m) => {
                    let payload = match m.payload_view::<str>() {
                        None => "",
                        Some(Ok(s)) => s,
                        Some(Err(e)) => {
                            eprintln!("Error while deserializing message payload: {:?}", e);
                            ""
                        }
                    };
                    println!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                          m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());

                    println!("{m:?}");

                    if let Some(headers) = m.headers() {
                        if let Some(request_id) = headers.iter().find(|header|{header.key == "request_id"}) {
                            println!("request_id: {}", String::from_utf8_lossy(request_id.value.unwrap_or(&[])));
                        }
                    }
                    consumer.commit_message(&m, CommitMode::Async).unwrap();
                }
            }
        }
    }

}
