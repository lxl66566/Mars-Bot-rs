pub mod constant;
pub use constant::*;
use url::Url;

pub fn msg_url(chat_id: i64, msg_id: i32) -> Result<Url, url::ParseError> {
    TELEGRAM_URL
        .join(chat_id.to_string().as_str())?
        .join(msg_id.to_string().as_str())
}
