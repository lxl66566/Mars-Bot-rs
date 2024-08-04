pub mod constant;
// pub mod telegram;
use std::sync::OnceLock;

pub use constant::*;
// pub use telegram::*;

/// TODO: deal with private group
pub fn msg_url(chat_invite_link: Option<&str>, chat_id: i64, msg_id: i32) -> String {
    let base = chat_invite_link.map_or_else(
        || {
            TELEGRAM_URL.to_owned().urljoin("c").urljoin(
                chat_id
                    .to_string()
                    .strip_prefix("-100")
                    .unwrap_or(&chat_id.to_string()),
            )
        },
        std::borrow::ToOwned::to_owned,
    );
    base.urljoin(msg_id.to_string())
}

pub trait OnceLockDefaultInit<T> {
    fn get_or_init_default(&self) -> &T;
}

impl<T: Default> OnceLockDefaultInit<T> for OnceLock<T> {
    fn get_or_init_default(&self) -> &T {
        self.get_or_init(Default::default)
    }
}

pub trait UrlJoin {
    fn urljoin(self, path: impl AsRef<str>) -> Self;
}

impl UrlJoin for String {
    fn urljoin(mut self, path: impl AsRef<str>) -> Self {
        if !self.ends_with('/') {
            self.push('/');
        }
        self.push_str(path.as_ref().trim_matches(|x| x == '/' || x == ' '));
        self
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_url_join() {
        let mut url = "https://example.com".to_string();
        assert_eq!(url.clone().urljoin("api/v1"), "https://example.com/api/v1");
        assert_eq!(url.clone().urljoin("/api/v1"), "https://example.com/api/v1");
        assert_eq!(
            url.clone().urljoin(" /api/v1/"),
            "https://example.com/api/v1"
        );
        url.push('/');
        assert_eq!(url.clone().urljoin("api/v1"), "https://example.com/api/v1");
        assert_eq!(url.clone().urljoin("/api/v1"), "https://example.com/api/v1");
        assert_eq!(
            url.clone().urljoin(" /api/v1/"),
            "https://example.com/api/v1"
        );
    }
}
