mod komoot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http_client = reqwest::Client::new();
    let komoot_api = komoot::ApiContext::new("https://api.komoot.de", &http_client);

    let kc = komoot_api.auth("", "").await?;

    match kc.user_context {
        Some(ct) => println!("{:?}", ct),
        None => println!("Something is wrong"),
    }

    Ok(())
}
