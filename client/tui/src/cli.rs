use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(long, default_value = "127.0.0.1")]
    pub ip: String,

    #[arg(long, default_value = "38800")]
    pub port: String,

    #[arg(long)]
    pub assets: Option<PathBuf>,
}
