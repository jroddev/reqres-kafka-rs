# reqres-kafka-rs
reqres-kafka-rs is a way of running the request-response pattern (HTTP) over the top of PubSub (Kafka)

## Requirements
- Requires rust toolchain including cargo
- Requires cmake to be installed to build lib-rdkafka
- Requires docker compose to run Kafka.

## Running
- sudo docker-compose up

Two APIs can then be tested through the proxy
- `curl -X POST -H "Content-Type: plain/text" --data "testing upper" http://localhost:3000/upper`
- `curl -X POST -H "Content-Type: plain/text" --data "testing reverse" http://localhost:3000/reverse`
If you want to hit the rest-api directly instead of via the Kafka proxies then you can change the port number from 3000 to 8080.

## Debugging
Kafdrop UI can be viewed at http://localhost:9000

## Performance
### single request
- direct to 8080: `2ms`
- via proxy 3000: `14-17ms`

This was tested via Postman and I did observe extra latency on the first request after being idle for a while

## Architecture
From a high level we have a pair of proxies on each end of Kafka. The request proxy takes in http requests, assigns a request_id, stores this in a map and publishes the request to Kafka. It will then listen to responses sent back through another Topic, pairs it map to the original request (request_id stored in Kafka Headers) and returns back to the user.
![Blank diagram(19)](https://user-images.githubusercontent.com/39473775/193689706-9fa52ece-24f3-4841-ae46-8af3cc550a97.png)

Three parallel systems are running to make this work:
- Axum http server to handle user request
- Kafka consumer thread to listen for responses
- Sync thread to store the request mappings
![Blank diagram(20)](https://user-images.githubusercontent.com/39473775/193689768-838129da-01db-4872-a789-baed9ce14fa5.png)
