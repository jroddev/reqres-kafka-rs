
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let resp = client.post("http://localhost:8080/upper")
        .body("Hello from the client")
        .send()
        .await?
        .text()
        .await?;
    println!("{:#?}", resp);
    Ok(())
}
