use camino::Utf8PathBuf;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub mqtt_broker_addr: String,
    pub mqtt_broker_port: u16,
    // pub mqtt_client_id: String, // TODO Unused because cloudmqtt does not yet have the interface
    pub mqtt_subscribe_prefix: String,

    pub initial_state_released: InitialState,
    pub initial_state_pressed: InitialState,
}

impl Config {
    pub async fn load(path_overwrite: Option<camino::Utf8PathBuf>) -> Result<Self, ConfigError> {
        let path = path_overwrite
            .map(Ok)
            .unwrap_or_else(Self::find_config_path_from_xdg)?;
        let config_contents = tokio::fs::read_to_string(&path).await?;

        toml::from_str(&config_contents).map_err(ConfigError::Toml)
    }

    fn find_config_path_from_xdg() -> Result<Utf8PathBuf, ConfigError> {
        let p = xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"))?
            .place_config_file("config.toml")?;

        camino::Utf8PathBuf::from_path_buf(p).map_err(ConfigError::NonUtf8Path)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Non-UTF8-Path: {}", .0.display())]
    NonUtf8Path(std::path::PathBuf),

    #[error("xdg error")]
    Xdg(#[from] xdg::BaseDirectoriesError),

    #[error("toml error")]
    Toml(#[source] toml::de::Error),
}

#[derive(Debug, serde::Deserialize)]
pub struct InitialState {
    pub row_0: [LedState; 5],
    pub row_1: [LedState; 5],
    pub row_2: [LedState; 5],
    pub row_3: [LedState; 5],
    pub row_4: [LedState; 5],
}

#[derive(Debug, serde::Deserialize)]
pub struct LedState(pub [u8; 3]);

impl LedState {
    pub fn unpack(&self) -> [u8; 3] {
        self.0
    }
}
