use std::io::{Result, Error};
use std::net::{SocketAddr, IpAddr};
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crossbeam_channel::Receiver;
use log::{info, warn, error};
use mtcp_rs::{TcpManager, TcpCanceller, TcpListener, TcpConnection, TcpStream, TcpError};
use num_cpus::get as cpu_count;

pub trait Handler : Send + Sync {
    fn handle_request(&self, stream: TcpStream) -> Result<()>;
}

pub struct Server {
    manager: Rc<TcpManager>,
    listener: TcpListener,
    backlog: usize,
    threads: usize,
}

impl Server {
    pub fn bind(bind_addr: IpAddr, port: u16, backlog: Option<usize>, threads: Option<usize>) -> Result<Self> {
        let threads = threads.unwrap_or_else(cpu_count).clamp(2, 64);
        let backlog = backlog.unwrap_or(256).clamp(1, 16384);
        let manager = TcpManager::instance()?;
        let listener = TcpListener::bind(&manager, SocketAddr::new(bind_addr, port))?;

        Ok(Self {
            manager,
            listener,
            backlog,
            threads,
        })
    }

    pub fn canceller(&self) -> Result<TcpCanceller> {
        self.manager.canceller()
    }

    pub fn run(&mut self, handler: impl Handler + 'static) -> Result<()>{
        info!("Server listening on: {}", self.listener.local_addr().unwrap());

        let handler = Arc::new(handler);
        let (channel_tx, channel_rx) = crossbeam_channel::bounded::<TcpConnection>(self.backlog);
        let mut error: Option<Error> = None;
        let mut thread_handles = Vec::with_capacity(self.threads);

        for _n in 0..self.threads {
            let thread_receiver = channel_rx.clone();
            let thread_handler = handler.clone();
            thread_handles.push(thread::spawn(move || Self::thread_main(thread_receiver, thread_handler)));
        }

        while !self.manager.cancelled() {
            match self.listener.accept(Some(Duration::from_secs(300))) {
                Ok(connection) => {
                    info!("Connection received: {:?} -> {:?}", connection.local_addr(), connection.peer_addr());
                    if let Err(error) = channel_tx.send_timeout(connection, Duration::from_secs(30)) {
                        warn!("Failed to enqueue the connection: {:?}", error);
                    }
                },
                Err(error) => {
                    match error {
                        TcpError::Cancelled=> error!("Accept operation was cancelled!"),
                        TcpError::TimedOut => error!("Accept operation timed out!"),
                        TcpError::Failed(inner) => error!("Accept operation failed: {:?}", inner),
                        TcpError::Incomplete | TcpError::TooBig => unreachable!(),
                    }
                },
            }
        }

        drop(channel_tx);

        thread_handles.drain(..).for_each(|handle| {
            if let Err(err) = handle.join().expect("Failed to join worker thread!") {
                error.replace(err);
            }
        });

        error.map(Err).unwrap_or(Ok(()))
    }

    fn thread_main(receiver: Receiver<TcpConnection>, handler: Arc<impl Handler>) -> Result<()> {
        let manager = TcpManager::instance()?;
        loop {
            match receiver.recv() {
                Ok(connection) => {
                    info!("Received connection from: {} [{:?}]", connection.local_addr().unwrap(), thread::current().id());
                    match TcpStream::from(&manager, connection) {
                        Ok(stream) => {
                            if let Err(err) = handler.handle_request(stream) {
                                error!("Request failed: {:?}", err);
                            }
                        },
                        Err(err) => error!("Failed to initialize stream: {:?}", err),
                    }
                },
                Err(_) => return Ok(()),
            };
        };
    }
}


