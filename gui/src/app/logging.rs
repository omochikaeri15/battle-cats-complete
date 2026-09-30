use std::env;
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tracing::{debug, info, Level};
use tracing_subscriber::fmt::writer::{BoxMakeWriter, MakeWriterExt};
use tracing_subscriber::{fmt, EnvFilter};

use kore::common::dirs;

const LOG: &str = "log.txt";
const PREVIOUS_LOG: &str = "log.prev.txt";

fn find_override_file(cwd: &Path, names: &[&str]) -> Option<PathBuf> {
    names.iter().map(|name| cwd.join(name)).find(|path| path.exists())
}

fn open(path: &Path) -> Option<File> {
    OpenOptions::new().write(true).create(true).truncate(true).open(path).ok()
}

fn rotate(dir: &Path) -> PathBuf {
    let log_file = dir.join(LOG);

    if log_file.exists() {
        let _ = fs::rename(&log_file, dir.join(PREVIOUS_LOG));
    }

    log_file
}

pub(crate) fn init_logging(enable_logging: bool) {
    let cwd = env::current_dir().unwrap_or_default();

    let trace_file = find_override_file(&cwd, &["trace.txt", "trace"]);
    let debug_file = find_override_file(&cwd, &["debug.txt", "debug"]);

    let app_dir = dirs::data();

    let (log_level, filter_directive, override_file) = if let Some(path) = trace_file {
        (Level::TRACE, "info,gui=trace,kore=trace,nyanko=trace,zbus=error", Some(path))
    } else if let Some(path) = debug_file {
        (Level::DEBUG, "info,gui=debug,kore=debug,nyanko=debug,zbus=error", Some(path))
    } else if enable_logging {
        (Level::INFO, "info,zbus=error", None)
    } else {
        if let Some(dir) = app_dir {
            let _ = fs::remove_file(dir.join(LOG));
            let _ = fs::remove_file(dir.join(PREVIOUS_LOG));
        }
        return;
    };

    let mirror = app_dir.map(|dir| rotate(&dir)).and_then(|path| open(&path));
    let primary = override_file.and_then(|path| open(&path));

    let writer = match (primary, mirror) {
        (Some(primary), Some(mirror)) => BoxMakeWriter::new(Arc::new(primary).and(Arc::new(mirror))),
        (Some(file), None) | (None, Some(file)) => BoxMakeWriter::new(Arc::new(file)),
        (None, None) => return,
    };

    let filter = EnvFilter::new(filter_directive);

    let subscriber = fmt::Subscriber::builder()
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(filter)
        .with_writer(writer)
        .with_ansi(false)
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);

    info!("Tracing initialized at {} level", log_level);
    debug!("Active filter directive: {}", filter_directive);
}
