use std::cell::RefCell;
use std::ffi::OsStr;
use std::fs::{self, File, Metadata};
use std::io::{Error as IoError, Result as IoResult, ErrorKind};
use std::num::NonZeroUsize;
use std::path::{PathBuf, Path, Component};
use std::str::FromStr;
use std::time::Duration;

use lazy_static::lazy_static;
use log::{info, warn};
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
}

impl WebHandler {
    pub fn new(root_path: &Path) -> IoResult<Self> {
        info!("Public path: {:?}", root_path.as_os_str());
        let root_path = fs::canonicalize(root_path).map_err(|_err| IoError::new(ErrorKind::NotFound, "Public directory not found!"))?;
        Ok(Self {
            root_path,
        })
    }

    fn parse_request(&self, mut stream: TcpStream, buffer: &mut Vec<u8>) -> IoResult<()> {
        buffer.clear();
        match stream.read_all_timeout(buffer, Some(Duration::from_secs(15)), None, NonZeroUsize::new(1048576), header_is_complete) {
            Ok(_) => {
                let response = self.process_request(Request::try_from(&buffer[..])?);
                response.send(stream)
            },
            Err(error) => Err(error.into()),
        }
    }

    fn process_request(&self, request: Request) -> Response {
        let method = request.method();
        match method {
            Method::GET => self.create_response(request, true),
            Method::HEAD => self.create_response(request, false),
            _ => {
                warn!("Method {:?} is not allowed!", method);
                Self::error_method_not_allowed()
            },
        }
    }

    fn create_response(&self, request: Request, transmit_data: bool) -> Response {
        let request_path = request.path();
        if let Some(full_path) = Self::sanitize_path(request_path).map(|path| self.root_path.join(path)) {
            if let Ok(meta_data) = full_path.metadata() {
                if meta_data.is_file() {
                    Self::serve_file_response(&full_path, &meta_data, transmit_data)
                } else {
                    warn!("Directory listing is forbidden!");
                    Self::error_forbidden()
                }
            } else {
                warn!("Requested resource {:?} not found!", full_path);
                Self::error_not_found()
            }
        } else {
            warn!("Request path {:?} is invalid!", request_path);
            Self::error_not_found()
        }
    }

    fn serve_file_response(full_path: &Path, meta_data: &Metadata, transmit_data: bool) -> Response {
        info!("Sending file: {:?}", full_path);
        if transmit_data {
            match File::open(full_path) {
                Ok(file) => {
                    Response::from_file(StatusCode::Ok, file, ContentType::from_path(full_path))
                },
                Err(_) => {
                    warn!("File {:?} could not be read!", full_path);
                    Self::error_internal()
                },
            }
        } else {
            Response::new(StatusCode::Ok, Some(meta_data.len()), ContentType::from_path(full_path))
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
    fn handle_request(&self, stream: TcpStream) -> IoResult<()> {
        BUFFER.with(|buffer| {
            self.parse_request(stream, &mut buffer.borrow_mut())
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
