use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(short, long)]
    pub logging: Option<Level>,

    /// Duration for intervals. For example for blinking
    #[clap(long)]
    #[clap(value_parser = humantime::parse_duration, default_value = "1s")]
    pub interval: std::time::Duration,

    /// Optional file to log to
    #[clap(long = "config")]
    pub config_path: Option<camino::Utf8PathBuf>,
}

#[derive(Default, Debug, Copy, Clone, clap::ValueEnum)]
pub enum Level {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl From<Level> for tracing::metadata::Level {
    fn from(value: Level) -> Self {
        match value {
            Level::Error => tracing::metadata::Level::ERROR,
            Level::Warn => tracing::metadata::Level::WARN,
            Level::Info => tracing::metadata::Level::INFO,
            Level::Debug => tracing::metadata::Level::DEBUG,
            Level::Trace => tracing::metadata::Level::TRACE,
        }
    }
}
