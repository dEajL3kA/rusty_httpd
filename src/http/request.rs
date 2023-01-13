use lazy_static::{lazy_static};
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::str;
use std::str::Utf8Error;

use regex::Regex;

use super::method::{Method, MethodError};
use super::QueryString;
use super::Headers;

#[derive(Debug)]
pub struct Request<'buf> {
    method: Method,
    path: &'buf str,
    query: Option<QueryString<'buf>>,
    headers: Option<Headers<'buf>>,
}

impl<'buf> Request<'buf> {
    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn path(&self) -> &str {
        self.path
    }

    pub fn headers(&self) -> Option<&Headers> {
        self.headers.as_ref()
    }

    pub fn query(&self) -> Option<&QueryString> {
        self.query.as_ref()
    }
}

impl<'buf> TryFrom<&'buf [u8]> for Request<'buf> {
    type Error = ParseError;

    fn try_from(buf: &'buf [u8]) -> Result<Request<'buf>, Self::Error> {
        // break lines
        let mut lines = str::from_utf8(buf)?.lines();

        // split request line
        let mut request = lines.next().ok_or(ParseError::Request)?.split_ascii_whitespace();

        // parse request
        let method = request.next().ok_or(ParseError::Request)?;
        let (path, query) = split(request.next().ok_or(ParseError::Request)?, "?");
        let protocol = request.next().ok_or(ParseError::Request)?;

        // check protocol version
        if !parse_protocol_version(protocol).map_or(false, |ver| (ver.0 == 1) && (ver.1 < 2)) {
            return Err(ParseError::Protocol);
        }

        // parse method, request headers and query
        let method: Method = method.parse()?;
        let query = query.map(QueryString::try_from).and_then(Result::ok);
        let headers = Headers::try_from(lines).ok();

        Ok(Self {
            method,
            path,
            headers,
            query,
        })
    }
}

fn split<'a>(str: &'a str, pattern: &str) -> (&'a str, Option<&'a str>) {
    let mut parts = str.splitn(2, pattern);
    (parts.next().unwrap_or(str), parts.next())
}

fn get_next_word(request: &str) -> Option<(&str, &str)> {
    for (i, c) in request.chars().enumerate() {
        if c == ' ' || c == '\r' {
            return Some((&request[..i], &request[i + 1..]));
        }
    }
    None
}

fn parse_protocol_version(version: &str) -> Option<(u32, u32)> {
    lazy_static! {
        static ref HTTP_VERSION: Regex = Regex::new(r"^http/(\d+)\.(\d+)$").unwrap();
    }
    match HTTP_VERSION.captures(&version.to_ascii_lowercase()) {
        Some(captures) => {
            let major: u32 = captures.get(1).map_or("0", |m| m.as_str()).parse().unwrap_or(0);
            let minor: u32 = captures.get(2).map_or("0", |m| m.as_str()).parse().unwrap_or(0);
            if major > 0 { Some((major, minor)) } else { None }
        },
        None => None,
    }
}

pub enum ParseError {
    Request,
    Encoding,
    Protocol,
    Method,
}

impl ParseError {
    fn message(&self) -> &str {
        match self {
            Self::Request => "Invalid Request",
            Self::Encoding => "Invalid Encoding",
            Self::Protocol => "Invalid Protocol",
            Self::Method => "Invalid Method",
        }
    }
}

impl From<MethodError> for ParseError {
    fn from(_: MethodError) -> Self {
        Self::Method
    }
}

impl From<Utf8Error> for ParseError {
    fn from(_: Utf8Error) -> Self {
        Self::Encoding
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Error for ParseError {}
