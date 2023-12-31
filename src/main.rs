use clap::Parser;
use komoot::KomootOpts;

mod komoot;

#[derive(Debug, Parser)]
#[clap(name = "stramoot", version)]
pub struct Cli {
    #[clap(flatten)]
    pub komoot: KomootOpts,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let http_client = reqwest::Client::new();
    let api = komoot::ApiContext::new("https://api.komoot.de", &http_client)
        .auth(&cli.komoot.user_name, &cli.komoot.password)
        .await?;
    let tours = api
        .tours(
            chrono::Utc::now()
                .checked_sub_months(chrono::Months::new(6))
                .unwrap(),
            10,
        )
        .await?;

    match tours.first() {
        Some(tour) => {
            println!("Tour {:?}", tour);
            println!("Content len: {0}", api.download(tour.id).await?.len())
        }
        None => println!("No tours"),
    }

    println!("{:?}", tours);

    Ok(())
}
