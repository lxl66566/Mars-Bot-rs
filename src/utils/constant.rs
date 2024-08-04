use std::{path::PathBuf, sync::LazyLock};

use home::home_dir;

pub static TELEGRAM_URL: &str = "https://t.me/";

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
