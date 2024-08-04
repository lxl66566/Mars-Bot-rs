use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// only reply mars warning if the message is from a channel.
    pub only_mars_for_channel_message: bool,
    pub token: Option<String>,
    /// allowed max file size in bytes.
    pub max_file_size: u32,
    /// Mars prompt. The origin message link will be filled in `{}`.
    ///
    /// The prompt should be formatted as markdown. Additional escape rule: <https://core.telegram.org/bots/api#formatting-options>.
    /// If you find some error like `Character '.' is reserved and must be
    /// escaped...`, please escape them.
    pub mars_prompt: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
            only_mars_for_channel_message: false,
            token: None,
            mars_prompt: "You Marsed\\! [Origin message]({})".to_string(),
        }
    }
}
