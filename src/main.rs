use std::ops::Sub;

use clap::Parser;
use cli::Cli;
use futures::StreamExt;
use komoot::models::Tour;

mod cli;
mod komoot;
mod strava;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();
    let http = reqwest::Client::new();

    let strava = strava::api::ApiContext::new(&http)
        .auth(
            &cli.strava.client_id,
            &cli.strava.client_secret,
            &cli.strava.refresh_token,
        )
        .await?;

    let komoot = komoot::api::ApiContext::new(&http)
        .auth(&cli.komoot.user_name, &cli.komoot.password)
        .await?;

    sync(&cli, &komoot, &strava).await
}

async fn sync(
    cli: &Cli,
    src: &komoot::api::ApiContext,
    dest: &strava::api::ApiContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let tours = src.tours(chrono::Utc::now().sub(cli.interval)).await?;

    let results: Vec<_> = futures::stream::iter(tours)
        .map(|t| sync_tour(t, &src, &dest))
        .buffered(3)
        .collect()
        .await;

    results.into_iter().for_each(|r| match r {
        Ok(id) => println!("Tour {} uploaded", id),
        Err(e) => eprintln!("Processing error. {}", e),
    });

    Ok(())
}

async fn sync_tour(
    tour: Tour,
    src: &komoot::api::ApiContext,
    dest: &strava::api::ApiContext,
) -> Result<u32, Box<dyn std::error::Error>> {
    let status = dest
        .upload(&tour.id.to_string(), &tour.name, src.stream(tour.id).await?)
        .await?;

    dest.wait_for_upload(status.id, 10, chrono::Duration::seconds(1))
        .await?;

    Ok(tour.id)
}
