use std::{fmt::Display, path::Path};
use case_insensitive_hashmap::CaseInsensitiveHashMap;

use lazy_static::lazy_static;

#[derive(Copy, Clone)]
pub enum ContentType {
    Binary,
    BZip2,
    CSS,
    GIF,
    GZip,
    HTML,
    JavaScript,
    JPEG,
    PDF,
    PNG,
    Tar,
    Text,
    ZIP,
    XZ,
}

lazy_static! {
    static ref EXTENSIONS: CaseInsensitiveHashMap<ContentType> = {
        let mut builder = CaseInsensitiveHashMap::new();
        builder.insert("bin", ContentType::Binary);
        builder.insert("bz2", ContentType::BZip2);
        builder.insert("css", ContentType::CSS);
        builder.insert("dat", ContentType::Binary);
        builder.insert("exe", ContentType::Binary);
        builder.insert("gif", ContentType::GIF);
        builder.insert("gz", ContentType::GZip);
        builder.insert("htm", ContentType::HTML);
        builder.insert("html", ContentType::HTML);
        builder.insert("jpe", ContentType::JPEG);
        builder.insert("jpeg", ContentType::JPEG);
        builder.insert("jpg", ContentType::JPEG);
        builder.insert("js", ContentType::JavaScript);
        builder.insert("pdf", ContentType::PDF);
        builder.insert("png", ContentType::PNG);
        builder.insert("tar", ContentType::Tar);
        builder.insert("tbz2", ContentType::BZip2);
        builder.insert("tgz", ContentType::GZip);
        builder.insert("txt", ContentType::Text);
        builder.insert("txz", ContentType::XZ);
        builder.insert("xz", ContentType::XZ);
        builder.insert("zip", ContentType::ZIP);
        builder
    };
}

impl ContentType {
    pub fn from_path(path: &Path) -> Option<ContentType> {
        match path.extension() {
            Some(ext) => match ext.to_str() {
                Some(str) => EXTENSIONS.get(str).copied(),
                None => None,
            },
            None => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::Binary => "application/octet-stream",
            Self::CSS => "text/css",
            Self::GIF => "image/gif",
            Self::GZip => "application/gzip",
            Self::BZip2 => "application/x-bzip2",
            Self::HTML => "text/html",
            Self::JavaScript => "text/javascript",
            Self::JPEG => "image/jpg",
            Self::PDF => "application/pdf",
            Self::PNG => "image/png",
            Self::Tar => "application/x-tar",
            Self::Text => "text/plain",
            Self::ZIP => "application/zip",
            Self::XZ => "application/x-xz",
        }
    }
}

impl AsRef<str> for ContentType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
