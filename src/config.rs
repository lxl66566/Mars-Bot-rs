use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Config {
    /// only reply mars warning if the message is from a channel.
    pub only_mars_for_channel_message: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            only_mars_for_channel_message: false,
        }
    }
}
