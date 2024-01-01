use clap::{self, error::Error, error::ErrorKind, Args, Parser};

#[derive(Debug, Parser)]
#[clap(name = "stramoot", version)]
pub struct Cli {
    #[clap(flatten)]
    pub komoot: KomootOpts,

    #[clap(short = 'i', long = "interval", value_parser = parse_iso8601, default_value = "P2DT")]
    pub interval: std::time::Duration,

    #[clap(short = 'l', long = "limit")]
    pub limit: u16,
}

fn parse_iso8601(duration: &str) -> Result<std::time::Duration, clap::Error> {
    iso8601_duration::Duration::parse(duration)
        .map_err(|_| Error::new(ErrorKind::InvalidValue))?
        .to_std()
        .ok_or(Error::new(ErrorKind::InvalidValue))
}

#[derive(Debug, Args)]
pub struct KomootOpts {
    #[clap(short = 'u', long = "user-name")]
    pub user_name: String,

    #[clap(short = 'p', long = "password")]
    pub password: String,
}
