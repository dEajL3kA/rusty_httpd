use std::fmt::{Debug, Write};
use std::fs::File;
use std::io::{Result as IoResult, Read};
use std::time::Duration;

use mtcp_rs::{TcpStream, TcpError};

use super::StatusCode;
use super::content_type::ContentType;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
enum Body {
    File(File),
    Str(&'static str),
    String(String),
    Buffer(Vec<u8>),
    None,
}

#[derive(Debug)]
pub struct Response { 
    header: String,
    body: Body,
}

impl Response {
    pub fn new(status_code: StatusCode, size: Option<u64>, content_type: Option<ContentType>) -> Self {
        Self {
            header: Self::create_header(status_code, size, content_type),
            body: Body::None,
        }
    }

    pub fn from_file(status_code: StatusCode, file: File, content_type: Option<ContentType>) -> Self {
        Self {
            header: Self::create_header(status_code, file_size(&file), content_type),
            body: Body::File(file),
        }
    }

    pub fn from_text(status_code: StatusCode, text: &'static str, content_type: Option<ContentType>) -> Self {
        Self {
            header: Self::create_header(status_code, Some(text.len() as u64), content_type.or(Some(ContentType::Text))),
            body: Body::Str(text),
        }
    }

    pub fn from_string(status_code: StatusCode, string: String, content_type: Option<ContentType>) -> Self {
        Self {
            header: Self::create_header(status_code, Some(string.len() as u64), content_type.or(Some(ContentType::Text))),
            body: Body::String(string),
        }
    }

    pub fn from_data(status_code: StatusCode, data: Vec<u8>, content_type: Option<ContentType>) -> Self {
        Self {
            header: Self::create_header(status_code, Some(data.len() as u64), content_type),
            body: Body::Buffer(data),
        }
    }

    fn create_header(status_code: StatusCode, length: Option<u64>, content_type: Option<ContentType>) -> String {
        let mut header = String::with_capacity(150);
        write!(header, "HTTP/1.1 {} {}\r\n", status_code, status_code.reason_phrase()).unwrap();
        write!(header, "Server: Rusty HTTP Server {}\r\n", PKG_VERSION).unwrap();
        if let Some(len) = length {
            write!(header, "Content-Length: {}\r\n", len).unwrap();
        }
        if let Some(ctype) = content_type {
            write!(header, "Content-Type: {}\r\n", ctype.as_ref()).unwrap();
        }
        write!(header, "\r\n").unwrap();
        header
    }

    pub fn send(self, mut writer: TcpStream) -> IoResult<()> {
        writer.write_all_timeout(self.header.as_bytes(), Some(Duration::from_secs(15)))?;
        self.body.send(writer)
    }
}

impl Body {
    pub fn send(self, writer: TcpStream) -> IoResult<()> {
        match self {
            Self::File(file) => Self::transfer_from_file(writer, file),
            Self::Str(str) => Self::transfer(writer, str.as_bytes()),
            Self::String(string) => Self::transfer(writer, string.as_bytes()),
            Self::Buffer(buffer) => Self::transfer(writer, &buffer[..]),
            Self::None => Ok(()),
        }
    }

    fn transfer(mut writer: TcpStream, source: &[u8]) -> IoResult<()> {
        writer.write_all_timeout(source, Some(Duration::from_secs(15))).map_err(TcpError::into)
    }

    fn transfer_from_file(mut writer: TcpStream, mut source: impl Read + 'static) -> IoResult<()> {
        let mut temp = [0u8; 4096];
        loop {
            match source.read(&mut temp)? {
                0 => return Ok(()),
                length => writer.write_all_timeout(&temp[0..length], Some(Duration::from_secs(15)))?,
            }
        }
    }
}

fn file_size(file: &File) -> Option<u64> {
    file.metadata().map(|metadata| metadata.len()).ok()
}
