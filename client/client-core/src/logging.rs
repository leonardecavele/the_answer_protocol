use super::errors::ClientError;
use std::fs::{self, OpenOptions};
use std::io::{self, IsTerminal, Write};
use std::sync::{Arc, Mutex};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, fmt};

const MAX_LOG_SIZE: u64 = 5 * 1024 * 1024;
const MAX_LOG_GENERATIONS: u32 = 5;
const DEFAULT_LOG_FILTER: &str = "warn,client_core=debug,client_api=debug";

struct DeferredStderr(Arc<Mutex<Vec<u8>>>);

impl Write for DeferredStderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut buffer = self
            .0
            .lock()
            .map_err(|_| io::Error::other("stderr buffer mutex poisoned"))?;

        buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct DeferredErrors(Arc<Mutex<Vec<u8>>>);

impl DeferredErrors {
    pub fn flush(&self) -> io::Result<()> {
        let mut buffer = self
            .0
            .lock()
            .map_err(|_| io::Error::other("stderr buffer mutex poisoned"))?;

        if buffer.is_empty() {
            return Ok(());
        }

        io::stderr().write_all(&buffer)?;
        buffer.clear();

        Ok(())
    }
}

pub fn setup(path: &str) -> Result<DeferredErrors, ClientError> {
    rotate(path);

    let file = OpenOptions::new().create(true).append(true).open(path)?;

    let buffer = Arc::new(Mutex::new(Vec::new()));
    let is_stderr_terminal = io::stderr().is_terminal();
    let sink = buffer.clone();

    let deferred_layer = is_stderr_terminal.then(|| {
        fmt::layer()
            .with_writer(move || DeferredStderr(sink.clone()))
            .with_ansi(false)
            .with_filter(LevelFilter::ERROR)
    });

    let immediate_layer = (!is_stderr_terminal).then(|| {
        fmt::layer()
            .with_writer(io::stderr)
            .with_ansi(false)
            .with_filter(LevelFilter::ERROR)
    });

    tracing_subscriber::registry()
        .with(
            fmt::layer().with_writer(file).with_ansi(false).with_filter(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_FILTER)),
            ),
        )
        .with(deferred_layer)
        .with(immediate_layer)
        .init();

    Ok(DeferredErrors(buffer))
}

fn rotate(path: &str) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };

    if metadata.len() < MAX_LOG_SIZE {
        return;
    }

    let mut generation = 1;

    while generation < MAX_LOG_GENERATIONS
        && fs::metadata(format!("{}.{}", path, generation)).is_ok()
    {
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
