/*
 * Rusty HTTP Server - simple and scalable HTTP server
 * This is free and unencumbered software released into the public domain.
 */
use std::cell::RefCell;
use std::env;
use std::ffi::OsStr;
use std::fs::{File, Metadata};
use std::io::{Error as IoError, Result as IoResult, ErrorKind};
use std::num::NonZeroUsize;
use std::path::{PathBuf, Path, Component};
use std::str::FromStr;
use std::time::Duration;

use lazy_static::lazy_static;
use log::{trace, debug, info, warn, log_enabled, Level};
use mtcp_rs::TcpStream;
use regex::bytes::Regex;

use crate::http::content_type::ContentType;
use crate::http::{ParseError, Method};
use crate::server::Handler;
use crate::http::{Response, StatusCode, Request};

thread_local! {
    static BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

pub struct WebHandler {
    root_path: PathBuf,
    timeout: Option<Duration>,
}

impl WebHandler {
    pub fn new(root_path: &Path, timeout: Option<Duration>) -> IoResult<Self> {
        let root_path = absolute(root_path)?;
        info!("Document root: {:?}", root_path);
        if !root_path.is_dir() {
            return Err(IoError::new(ErrorKind::NotFound, "Root directory not found!"));
        }
        Ok(Self {
            root_path,
            timeout,
        })
    }

    fn parse_request(&self, id: usize, mut stream: TcpStream, buffer: &mut Vec<u8>) -> IoResult<()> {
        buffer.clear();
        match stream.read_all_timeout(buffer, self.timeout, None, NonZeroUsize::new(1048576), header_is_complete) {
            Ok(_) => {
                let response = self.process_request(id, Request::try_from(&buffer[..])?);
                response.send(stream, self.timeout)
            },
            Err(error) => Err(error.into()),
        }
    }

    fn process_request(&self, id: usize, request: Request) -> Response {
        let request_method = request.method();
        if !log_enabled!(Level::Trace) {
            debug!("[id:{id:X}] Request: {request_method} {:?}", request.path());
        } else {
            trace!("[id:{id:X}] {:?}", request);
        }
        match request_method {
            Method::GET => self.create_response(id, request, true),
            Method::HEAD => self.create_response(id, request, false),
            _ => {
                warn!("[id:{id:X}] Method {:?} is not allowed!", request_method);
                Self::error_method_not_allowed()
            },
        }
    }

    fn create_response(&self, id: usize, request: Request, transmit_data: bool) -> Response {
        let request_path = request.path();
        if let Some(full_path) = Self::sanitize_path(request_path).map(|path| self.root_path.join(path)) {
            if let Ok(file_info) = full_path.metadata() {
                trace!("[id:{id:X}] File meta information: {:?}", file_info);
                if !file_info.is_dir() {
                    Self::serve_file_response(id, &full_path, &file_info, transmit_data)
                } else {
                    warn!("[id:{id:X}] Directory listing is forbidden!");
                    Self::error_forbidden()
                }
            } else {
                warn!("[id:{id:X}] Requested resource {:?} could not be found!", full_path);
                Self::error_not_found()
            }
        } else {
            warn!("[id:{id:X}] Request path {:?} is invalid!", request_path);
            Self::error_not_found()
        }
    }

    fn serve_file_response(id: usize, full_path: &Path, file_info: &Metadata, transmit_data: bool) -> Response {
        match File::open(full_path) {
            Ok(file) => if transmit_data {
                info!("[id:{id:X}] Sending file: {:?} (size: {:?} bytes)", full_path, file_info.len());
                Response::from_file(StatusCode::Ok, file, ContentType::from_path(full_path))
            } else {
                info!("[id:{id:X}] File content-length is: {:?} bytes", file_info.len());
                Response::new(StatusCode::Ok, Some(file_info.len()), ContentType::from_path(full_path))
            },
            Err(_) => {
                warn!("[id:{id:X}] File {:?} could not be opened!", full_path);
                Self::error_internal()
            },
        }
    }

    fn sanitize_path(path_str: &str) -> Option<PathBuf> {
        static DELIM: [ char; 2 ] = [ '/', '\\' ];
        let iterator = Path::new(path_str.trim_start_matches(DELIM)).components();
        let mut components = Vec::with_capacity(iterator.clone().count());

        for component in iterator {
            match component {
                Component::Normal(normal) => components.push(Self::check_filename(normal)?),
                Component::CurDir => {},
                Component::ParentDir => _ = components.pop()?,
                Component::Prefix(_) | Component::RootDir => return None,
            }
        }

        (!components.is_empty())
            .then(|| components.into_iter().collect::<PathBuf>())
            .or_else(|| PathBuf::from_str("index.html").ok())
    }

    fn check_filename(name: &OsStr) -> Option<&OsStr> {
        static ILLEGAL_CHARS: [ char; 9 ] = [ '<', '>', ':', '"', '/', '\\', '|', '?', '*' ];
        let name_str = name.to_str()?;
        (!(name_str.contains(ILLEGAL_CHARS) || name_str.starts_with('.'))).then_some(name)
    }

    fn error_forbidden() -> Response {
        const HTML_TEXT: &str = "<!doctype html><title>Error 403</title><h1>403 Forbidden</h1><h3>You don't have permission to access the requested resource on this server.</h3>\n";
        Response::from_text(StatusCode::Forbidden, HTML_TEXT, Some(ContentType::HTML))
    }

    fn error_not_found() -> Response {
        const HTML_TEXT: &str = "<!doctype html><title>Error 404</title><h1>404 Not Found</h1><h3>The requested resource could not be found on this server, but may be available in the future.</h3>\n";
        Response::from_text(StatusCode::NotFound, HTML_TEXT, Some(ContentType::HTML))
    }

    fn error_method_not_allowed() -> Response {
        const HTML_TEXT: &str = "<!doctype html><title>Error 405</title><h1>405 Method Not Allowed</h1><h3>The request method is known by the server, but is not supported by the target resource.</h3>\n";
        Response::from_text(StatusCode::MethodNotAllowed, HTML_TEXT, Some(ContentType::HTML))
    }

    fn error_internal() -> Response {
        const HTML_TEXT: &str = "<!doctype html><title>Error 500</title><h1>500 Internal Server Error</h1><h3>The server encountered an internal error and was unable to complete your request.</h3>\n";
        Response::from_text(StatusCode::InternalServerError, HTML_TEXT, Some(ContentType::HTML))
    }
}

impl Handler for WebHandler {
    fn handle_request(&self, id: usize, stream: TcpStream) -> IoResult<()> {
        BUFFER.with(|buffer| {
            self.parse_request(id, stream, &mut buffer.borrow_mut())
        })
    }
}

impl From<ParseError> for IoError {
    fn from(err: ParseError) -> Self {
        IoError::new(ErrorKind::InvalidData, err)
    }
}

fn header_is_complete(buffer: &[u8]) -> bool {
    lazy_static! {
        static ref END_MARKER: Regex = Regex::new(r"\x0D\x0A[\x09\x0B\x0C\x20]*\x0D\x0A").unwrap();
    }
    END_MARKER.is_match(buffer)
}

pub fn absolute(path: &Path) -> IoResult<PathBuf> {
    if !path.is_absolute() {
        Ok(env::current_dir()?.join(path))
    } else {
        Ok(path.to_path_buf())
    }
}
