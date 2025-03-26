use camino::Utf8PathBuf;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub mqtt_broker_addr: String,
    pub mqtt_broker_port: u16,
    // pub mqtt_client_id: String, // TODO Unused because cloudmqtt does not yet have the interface
    pub mqtt_subscribe_prefix: String,

    pub keypad: KeypadConfig,
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
pub struct KeypadConfig {
    pub pad_0_0: PadConfig,
    pub pad_0_1: PadConfig,
    pub pad_0_2: PadConfig,
    pub pad_0_3: PadConfig,
    pub pad_0_4: PadConfig,
    pub pad_1_0: PadConfig,
    pub pad_1_1: PadConfig,
    pub pad_1_2: PadConfig,
    pub pad_1_3: PadConfig,
    pub pad_1_4: PadConfig,
    pub pad_2_0: PadConfig,
    pub pad_2_1: PadConfig,
    pub pad_2_2: PadConfig,
    pub pad_2_3: PadConfig,
    pub pad_2_4: PadConfig,
    pub pad_3_0: PadConfig,
    pub pad_3_1: PadConfig,
    pub pad_3_2: PadConfig,
    pub pad_3_3: PadConfig,
    pub pad_3_4: PadConfig,
    pub pad_4_0: PadConfig,
    pub pad_4_1: PadConfig,
    pub pad_4_2: PadConfig,
    pub pad_4_3: PadConfig,
    pub pad_4_4: PadConfig,
}

#[derive(Debug, serde::Deserialize)]
pub struct PadConfig {
    pub released: [u8; 3],
    pub pressed: [u8; 3],
    pub on_press: Vec<OnPressAction>,
    pub on_release: Vec<OnReleaseAction>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub enum OnPressAction {
    ToggleBlinking,

    Publish {
        topic: String,
        payload: String,
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub enum OnReleaseAction {
    Publish {
        topic: String,
        payload: String,
    }
}
