
## Requirements
Requires rust toolchain including cargo
Requires cmake to be installed to build lib-rdkafka
Requires docker compose to run Kafka.

## Running
At the moment the apps need to be run individually
- sudo docker-compose up
- cargo run --release --bin rest-api
- cargo run --release --bin response-proxy
- cargo run --release --bin request-proxy

Two APIs can then be tested through the proxy
- `curl -X POST -H "Content-Type: plain/text" --data "testing upper" http://localhost:3000/upper`
- `curl -X POST -H "Content-Type: plain/text" --data "testing reverse" http://localhost:3000/reverse`
If you want to hit the rest-api directly instead of via the Kafka proxies then you can change the port number from 3000 to 8080.

## Debugging
Kafdrop UI can be viewed at http://localhost:9000

## Performance
### single request
- direct to 8080: 2ms
- via proxy 3000: 14-17ms

This was tested via Postman and I did observe extra latency on the first request after being idle for a while
