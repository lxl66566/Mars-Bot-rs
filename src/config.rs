use std::sync::OnceLock;

use serde::{Deserialize, Serialize};


pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// only reply mars warning if the message is from a channel.
    pub only_mars_for_channel_message: bool,
    pub token: Option<String>,
}
