use core::str::Lines;
use std::convert::TryFrom;
use std::io::{Error as IoError, ErrorKind};
use crate::utils::ValueMap;

#[derive(Debug)]
pub struct Headers<'buf> {
    map: ValueMap<'buf>,
}

impl<'buf> Headers<'buf> {
    pub fn names(&'buf self) -> impl Iterator<Item = &'buf str> {
        self.map.keys()
    }

    pub fn values(&'buf self, name: &str) -> Option<impl Iterator<Item = &str>> {
        self.map.values(name)
    }
}

impl<'buf> TryFrom<Lines<'buf>> for Headers<'buf> {
    type Error = IoError;

    fn try_from(lines: Lines<'buf>) -> Result<Self, IoError> {
        let mut map = ValueMap::new();

        for line in lines.map(str::trim).take_while(|&str| !str.is_empty()) {
            let mut parts = line.splitn(2, ':');
            let key = parts.next().map(str::trim).unwrap_or_default();
            let val = parts.next().map(str::trim).unwrap_or_default();

            if !key.is_empty() {
                map.put(key.into(), val.into());
            }
        }

        if map.is_empty() {
            return Err(IoError::new(ErrorKind::NotFound, "No headers found!"));
        }

        Ok (Self { map })
    }
}
