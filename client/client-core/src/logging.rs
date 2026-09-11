use super::errors::ClientError;
use std::fs::{self, OpenOptions};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, fmt};

const MAX_LOG_SIZE: u64 = 5 * 1024 * 1024;

pub fn setup(path: &str) -> Result<(), ClientError> {
    rotate(path);

    let file = OpenOptions::new().create(true).append(true).open(path)?;

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(file).with_ansi(false).with_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")),
        ))
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_filter(LevelFilter::ERROR),
        )
        .init();

    Ok(())
}

fn rotate(path: &str) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };

    if metadata.len() < MAX_LOG_SIZE {
        return;
    }

    let mut generation = 1;

    while fs::metadata(format!("{}.{}", path, generation)).is_ok() {
        generation += 1;
    }

    while generation > 1 {
        let _ = fs::rename(
            format!("{}.{}", path, generation - 1),
            format!("{}.{}", path, generation),
        );

        generation -= 1;
    }

    let _ = fs::rename(path, format!("{}.1", path));
}
