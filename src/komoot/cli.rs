use clap::{self, Args};

#[derive(Debug, Args)]
pub struct KomootOpts {
    #[clap(short = 'u', long = "user-name")]
    pub user_name: String,

    #[clap(short = 'p', long = "password")]
    pub password: String,
}
