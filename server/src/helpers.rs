use headers::{HeaderMap, HeaderValue};
use hyper::header::{AsHeaderName, IntoHeaderName};

pub trait HeaderMapUtils {
    fn get_str<K: AsHeaderName>(&self, key: K) -> Option<&str>;
    fn get_string<K: AsHeaderName>(&self, key: K) -> Option<String>;
    fn try_append<K: IntoHeaderName>(&mut self, key: K, value: Option<String>) -> &mut Self;
}

impl HeaderMapUtils for HeaderMap {
    fn get_str<K: AsHeaderName>(&self, key: K) -> Option<&str> {
        self.get(key).and_then(|v| v.to_str().ok())
    }
    fn get_string<K: AsHeaderName>(&self, key: K) -> Option<String> {
        self.get_str(key).map(|s| s.to_string())
    }
    fn try_append<K: IntoHeaderName>(&mut self, key: K, value: Option<String>) -> &mut Self {
        if let Some(value) = value.and_then(|etag| HeaderValue::from_str(etag.as_ref()).ok()) {
            self.append(key, value);
        }
        self
    }
}

pub trait MyIterUtils: std::marker::Sized {
    type Item;
    fn pipe<F, R>(self, fun: F) -> R
    where
        R: Iterator<Item = Self::Item>,
        F: FnOnce(Self) -> R;
}

impl<I, T> MyIterUtils for I
where
    I: Iterator<Item = T> + std::marker::Sized,
{
    type Item = T;
    fn pipe<F, R>(self, fun: F) -> R
    where
        R: Iterator<Item = Self::Item>,
        F: FnOnce(Self) -> R,
    {
        fun(self)
    }
}
