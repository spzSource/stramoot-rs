use std::ops::Sub;

use clap::Parser;

mod cli;
mod komoot;
mod strava;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();
    let http_client = reqwest::Client::new();

    let komoot = komoot::api::ApiContext::new("https://api.komoot.de", &http_client)
        .auth(&cli.komoot.user_name, &cli.komoot.password)
        .await?;
    let strava = strava::api::ApiContext::new(&http_client)
        .auth(
            &cli.strava.client_id,
            &cli.strava.client_secret,
            &cli.strava.refresh_token,
        )
        .await?;

    for tour in komoot
        .tours(chrono::Utc::now().sub(cli.interval), cli.limit)
        .await?
    {
        println!("Tour {:?}", tour);
        let content = komoot.download(tour.id).await?;
        println!("content length: {}", content.len());
        let status = strava
            .upload(&tour.id.to_string(), &tour.name, &content)
            .await?;
        println!("{:?}", status)
    }

    Ok(())
}
