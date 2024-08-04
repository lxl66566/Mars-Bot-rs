pub mod constant;
use std::sync::OnceLock;

pub use constant::*;
use url::Url;

pub fn msg_url(chat_id: i64, msg_id: i32) -> Result<Url, url::ParseError> {
    TELEGRAM_URL
        .join(chat_id.to_string().as_str())?
        .join(msg_id.to_string().as_str())
}

pub trait OnceLockDefaultInit<T> {
    fn get_or_init_default(&self) -> &T;
}

impl<T: Default> OnceLockDefaultInit<T> for OnceLock<T> {
    fn get_or_init_default(&self) -> &T {
        self.get_or_init(Default::default)
    }
}
