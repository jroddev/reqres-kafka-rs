
Requires cmake to be installed to build lib-rdkafka
Requires docker compose to run Kafka `sudo docker-compose up`
Kafdrop UI can be viewed at http://localhost:9000

At the moment the apps need to be run individually
- sudo docker-compose up
- cargo run --release --bin rest-api
- cargo run --release --bin response-proxy
- cargo run --release --bin request-proxy

Two APIs can then be tested
`curl -X POST -H "Content-Type: plain/text" --data "testing upper" http://localhost:3000/upper`
`curl -X POST -H "Content-Type: plain/text" --data "testing reverse" http://localhost:3000/reverse`
