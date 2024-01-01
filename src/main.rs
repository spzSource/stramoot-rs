use std::ops::Sub;

use clap::Parser;

mod cli;
mod komoot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();

    let http_client = reqwest::Client::new();
    let api = komoot::api::ApiContext::new("https://api.komoot.de", &http_client)
        .auth(&cli.komoot.user_name, &cli.komoot.password)
        .await?;
    let tours = api.tours(chrono::Utc::now().sub(cli.interval), cli.limit).await?;

    match tours.first() {
        Some(tour) => {
            println!("Tour {:?}", tour);
            println!("Content len: {0}", api.download(tour.id).await?.len())
        }
        None => println!("No tours"),
    }

    Ok(())
}
