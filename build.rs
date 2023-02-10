use winres::WindowsResource;

extern crate winres;

const PRODUCTNAME: &str = "Rusty HTTP Server";
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = WindowsResource::new();
        res.set_icon("etc/resources/server.ico");
        res.set("CompanyName", "dEajL3kA");
        res.set("FileDescription", PRODUCTNAME);
        res.set("FileVersion", PKG_VERSION);
        res.set("InternalName", "rusty_httpd");
        res.set("LegalCopyright", "This is free and unencumbered software released into the public domain.");
        res.set("OriginalFilename", "rusty_httpd.exe");
        res.set("ProductName", PRODUCTNAME);
        res.set("ProductVersion", PKG_VERSION);
        res.compile().unwrap();
    }
}
