use std::ops::Sub;

use clap::Parser;
use cli::Cli;
use futures::{stream, Future, StreamExt};
use komoot::models::Tour;

mod cli;
mod komoot;
mod strava;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();
    let http = reqwest::Client::new();
    let (src, dest) = futures::join!(komoot(&cli.komoot, &http), strava(&cli.strava, &http));

    sync(&cli, &src?, &dest?)
        .await
        .into_iter()
        .for_each(|sr| match sr {
            Ok(id) => println!("Tour {} uploaded", id),
            Err(e) => eprintln!("Processing error. {}", e),
        });

    Ok(())
}

fn komoot<'a>(
    opts: &'a cli::KomootOpts,
    http_client: &'a reqwest::Client,
) -> impl Future<Output = Result<komoot::api::ApiContext, Box<dyn std::error::Error>>> + 'a {
    komoot::api::ApiContext::auth(&opts.user_name, &opts.password, &http_client)
}

fn strava<'a>(
    opts: &'a cli::StravaOpts,
    http_client: &'a reqwest::Client,
) -> impl Future<Output = Result<strava::api::ApiContext, Box<dyn std::error::Error>>> + 'a {
    strava::api::ApiContext::auth(
        &opts.client_id,
        &opts.client_secret,
        &opts.refresh_token,
        &http_client,
    )
}

async fn sync(
    cli: &Cli,
    src: &komoot::api::ApiContext,
    dest: &strava::api::ApiContext,
) -> Vec<Result<u32, Box<dyn std::error::Error>>> {
    let start = chrono::Utc::now().sub(cli.interval);
    src.tours_stream(start, cli.batch_size)
        .flat_map(|result| stream::iter(res_to_vec(result)))
        .map(|tour| async { sync_tour(src, dest, &tour?).await })
        .buffer_unordered(cli.batch_size as usize)
        .collect::<Vec<_>>()
        .await
}

async fn sync_tour(
    src: &komoot::api::ApiContext,
    dest: &strava::api::ApiContext,
    tour: &Tour,
) -> Result<u32, Box<dyn std::error::Error>> {
    let content = src.download(tour.id).await?;

    let status = dest
        .upload(&tour.id.to_string(), &tour.name, &content)
        .await?;

    dest.wait_for_upload(status.id, 10, chrono::Duration::seconds(1))
        .await?;

    Ok(tour.id)
}

fn res_to_vec<T, E>(res: Result<Vec<T>, E>) -> Vec<Result<T, E>> {
    match res {
        Ok(vec) => vec.into_iter().map(Ok).collect(),
        Err(err) => vec![Err(err)],
    }
}
