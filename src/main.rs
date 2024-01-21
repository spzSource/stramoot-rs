use std::ops::Sub;

use clap::Parser;
use cli::{Cli, CommonOpts};
use futures::{stream, StreamExt};
use komoot::models::Tour;

mod cli;
mod komoot;
mod strava;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Cli {
        komoot,
        strava,
        common,
    } = cli::Cli::parse();

    let http = reqwest::Client::new();
    let (src, dest) = futures::join!(
        komoot::api::ApiContext::auth(&komoot.username, &komoot.password, &http),
        strava::api::ApiContext::auth(
            &strava.client_id,
            &strava.client_secret,
            &strava.refresh_token,
            &http,
        )
    );

    sync(&src?, &dest?, &common)
        .await
        .into_iter()
        .for_each(|sr| match sr {
            Ok(id) => println!("Tour {} uploaded", id),
            Err(e) => eprintln!("Processing error. {}", e),
        });

    Ok(())
}

async fn sync(
    src: &komoot::api::ApiContext,
    dest: &strava::api::ApiContext,
    opts: &CommonOpts,
) -> Vec<Result<u32, Box<dyn std::error::Error>>> {
    let start = chrono::Utc::now().sub(opts.interval);
    src.tours_stream(start, opts.batch_size)
        .flat_map(|result| stream::iter(res_to_vec(result)))
        .map(|tour| async { sync_tour(src, dest, &tour?).await })
        .buffer_unordered(opts.batch_size as usize)
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
