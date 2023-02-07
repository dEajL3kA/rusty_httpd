#![allow(dead_code)]
#![allow(clippy::upper_case_acronyms)]

use env_logger::Env;
use log::{info, warn, error};
use std::env;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::str::FromStr;

use crate::server::Server;
use crate::web::WebHandler;

mod http;
mod server;
mod utils;
mod web;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    env_logger::init_from_env(Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"));
    info!("Rusty HTTP Server [{}]", PKG_VERSION);

    let public_path = env::var("HTTP_PUBLIC_PATH").ok().map(PathBuf::from).unwrap_or_else(default_public_path);
    let address = env::var("HTTP_BIND_ADDRESS").ok().map(|str| IpAddr::from_str(&str).expect("Failed to parse bind address!")).unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    let port_number = env::var("HTTP_PORT_NUMBER").ok().map(|str| str.parse().expect("Failed to parse port number!")).unwrap_or(8080);
    let thread_count = env::var("HTTP_THREADS").ok().map(|str| str.parse().expect("Failed to parse number of threads!"));

    if !public_path.is_dir() {
        error!("Public path {:?} does not exist, is not a directory, or is inaccessible!", public_path);
        panic!("Failed to initialize public path!");
    }

    let handler = WebHandler::new(&public_path).expect("Failed to create web-handler instance!");
    let mut server = Server::bind(address, port_number, None, thread_count).expect("Failed to create the server!");

    let canceller = server.canceller().expect("Failed to create canceller!");
    drop(ctrlc::set_handler(move || {
        warn!("Server shutdown has been requested !!!");
        if canceller.cancel().is_err() {
            error!("Failed to cancel the server!");
        }
    }));

    server.run(handler).unwrap();
    info!("Shutting down application. Goodbye!");
}

fn default_public_path() -> PathBuf {
    let exe_file = env::current_exe().expect("Failed to determine executable file path!");
    exe_file.parent().expect("Failed to determine base directory!").join("public")
}
