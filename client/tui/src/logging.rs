use crate::errors::ApplicationError;
use std::fs::OpenOptions;
use tracing_subscriber::EnvFilter;

pub fn setup(path: &str) -> Result<(), ApplicationError> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")),
        )
        .with_writer(file)
        .with_ansi(false)
        .init();

    Ok(())
}
