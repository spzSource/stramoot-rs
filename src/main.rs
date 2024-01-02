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
    let http_client = reqwest::Client::new();

    let strava = strava::api::ApiContext::new(&http_client)
        .auth(
            &cli.strava.client_id,
            &cli.strava.client_secret,
            &cli.strava.refresh_token,
        )
        .await?;

    let komoot = komoot::api::ApiContext::new(&http_client)
        .auth(&cli.komoot.user_name, &cli.komoot.password)
        .await?;

    sync(&cli, &komoot, &strava).await
}

async fn sync(
    cli: &Cli,
    src: &komoot::api::ApiContext,
    dest: &strava::api::ApiContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let tours = src
        .tours(chrono::Utc::now().sub(cli.interval), cli.limit)
        .await?;

    let res: Vec<_> = futures::stream::iter(tours)
        .map(|t| sync_tour(&src, &dest, t))
        .buffered(3)
        .collect()
        .await;

    for r in res {
        match r {
            Err(e) => eprintln!("Processing error. {}", e),
            Ok(_) => (),
        }
    }

    Ok(())
}

async fn sync_tour(
    src: &komoot::api::ApiContext,
    dest: &strava::api::ApiContext,
    tour: Tour,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Tour: {:?}", tour);

    let content = src.download(tour.id).await?;

    let status = dest
        .upload(&tour.id.to_string(), &tour.name, &content)
        .await?;

    dest.wait_for_upload(status.id, 10, chrono::Duration::seconds(1))
        .await?;

    println!("Tour: {:?} uploaded.", tour);

    Ok(())
}
