use std::borrow::Cow;
use std::convert::TryFrom;
use std::io::{Error as IoError, ErrorKind};
use crate::utils::ValueMap;
use urlencoding::decode as url_decode;

#[derive(Debug)]
pub struct QueryString<'buf> {
    map: ValueMap<'buf>,
}

impl<'buf> QueryString<'buf>{
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.map.keys()
    }

    pub fn values(&self, key: &str) -> Option<impl Iterator<Item = &str>> {
        self.map.values(key)
    }
}

impl<'buf> TryFrom<&'buf str> for QueryString<'buf> {
    type Error = IoError;

    fn try_from(query_string: &'buf str) -> Result<Self, IoError> {
        let mut map = ValueMap::new();

        for sub_str in query_string.split('&').map(str::trim).filter(|&str| !str.is_empty()) {
            let mut parts = sub_str.splitn(2, '=');
            let key = decode(parts.next().map(str::trim).unwrap_or_default());
            let val = decode(parts.next().map(str::trim).unwrap_or_default());

            if !key.is_empty() {
                map.put(key, val);
            }
        }

        if map.is_empty() {
            return Err(IoError::new(ErrorKind::NotFound, "No query parameters found!"));
        }

        Ok(Self { map })
    }
}

fn decode(str: &str) -> Cow<str> {
    match url_decode(str) {
        Ok(decoded) => decoded,
        Err(_) => str.into()
    }
}
