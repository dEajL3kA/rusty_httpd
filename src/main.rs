/*
 * Rusty HTTP Server - simple and scalable HTTP server
 * This is free and unencumbered software released into the public domain.
 */
#![allow(dead_code)]
#![allow(clippy::upper_case_acronyms)]

use env_logger::Env;
use log::{info, warn, error, LevelFilter};
use std::env;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use crate::server::Server;
use crate::web::WebHandler;

mod http;
mod server;
mod utils;
mod web;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    env_logger::init_from_env(Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, LevelFilter::Info.as_str()));
    info!("Rusty HTTP Server [{}]", PKG_VERSION);

    let public_path = env::var("HTTP_PUBLIC_PATH").map_or_else(|_error| default_public_path(), PathBuf::from);
    let address = env::var("HTTP_BIND_ADDRESS").ok().map_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED), |str| IpAddr::from_str(&str).expect("Failed to parse bind address!"));
    let port_number = env::var("HTTP_PORT_NUMBER").ok().map_or(8080, |str| str.parse().expect("Failed to parse port number!"));
    let thread_count = env::var("HTTP_THREADS").ok().map(|str| str.parse().expect("Failed to parse number of threads!"));
    let io_timeout = env::var("HTTP_TIMEOUT").ok().map_or(15000, |str| str.parse().expect("Failed to parse the timeout value!"));

    let public_full_path = public_path.canonicalize().ok().and_then(|path| path.is_dir().then_some(path));
    if public_full_path.is_none() {
        error!("Public path {:?} does not exist, is not a directory, or is inaccessible!", public_path);
    }

    let handler = WebHandler::new(&public_full_path.expect("Public path not found!"), duration(io_timeout)).expect("Failed to create web-handler instance!");
    let mut server = Server::bind(address, port_number, None, thread_count).expect("Failed to create the server!");

    let canceller = server.canceller().expect("Failed to create canceller!");
    drop(ctrlc::set_handler(move || {
        warn!("Server shutdown has been requested!");
        if canceller.cancel().is_err() {
            error!("Failed to cancel the running server!");
        }
    }));

    server.run(handler).unwrap();
    info!("Shutting down application. Goodbye!");
}

fn default_public_path() -> PathBuf {
    let exe_file = env::current_exe().expect("Failed to determine executable file path!");
    exe_file.parent().expect("Failed to determine base directory!").join("public")
}

fn duration(duration: u64) -> Option<Duration> {
    (duration > 0).then(|| Duration::from_millis(duration))
}
