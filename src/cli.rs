use clap::{self, error::Error, error::ErrorKind, Args, Parser};

#[derive(Debug, Parser)]
#[clap(name = "stramoot", version)]
pub struct Cli {
    #[clap(flatten)]
    pub komoot: KomootOpts,

    #[clap(flatten)]
    pub strava: StravaOpts,

    #[clap(flatten)]
    pub common: CommonOpts,
}

fn parse_iso8601(duration: &str) -> Result<std::time::Duration, clap::Error> {
    iso8601_duration::Duration::parse(duration)
        .map_err(|_| Error::new(ErrorKind::InvalidValue))?
        .to_std()
        .ok_or(Error::new(ErrorKind::InvalidValue))
}

#[derive(Debug, Args)]
pub struct CommonOpts {
    #[clap(short = 'i', long = "interval", value_parser = parse_iso8601, default_value = "P2DT")]
    pub interval: std::time::Duration,

    #[clap(short = 'b', long = "batch-size", default_value = "10")]
    pub batch_size: u8,
}

#[derive(Debug, Args)]
pub struct KomootOpts {
    #[clap(long = "komoot-username")]
    pub username: String,

    #[clap(long = "komoot-password")]
    pub password: String,
}

#[derive(Debug, Args)]
pub struct StravaOpts {
    #[clap(long = "strava-client-id")]
    pub client_id: String,

    #[clap(long = "strava-client-secret")]
    pub client_secret: String,

    #[clap(long = "strava-refresh-token")]
    pub refresh_token: String,
}
