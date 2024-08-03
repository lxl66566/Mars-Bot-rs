use std::{path::PathBuf, sync::LazyLock};

use home::home_dir;
use url::Url;

pub static TELEGRAM_URL: LazyLock<Url> =
    LazyLock::new(|| Url::from_file_path("https://t.me/").expect("hardcoded url should be valid"));

pub static DATA_ROOT_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    home_dir()
        .expect("home path should exist")
        .join(".local")
        .join(env!("CARGO_BIN_NAME"))
});

pub fn config_path() -> PathBuf {
    DATA_ROOT_PATH.join("config.toml")
}

pub fn db_path() -> PathBuf {
    DATA_ROOT_PATH.join("db")
}
