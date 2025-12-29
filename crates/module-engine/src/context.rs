use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use graphics::graphics::Graphics;
use http::Request;
use http_body_util::Empty;
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::client::legacy::{Client, connect::HttpConnector};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpSocket, TcpStream, UdpSocket},
};

use crate::{config::Config, error, info, logging::Logger};

pub struct ExecutionContext {
    logger: Logger,
    capabilities: HashSet<String>,
    config: Config,

    //Http
    http_request_next_id: i32,
    requests: HashMap<i32, http::request::Builder>,
    prepared_requests: HashMap<i32, Request<Empty<Bytes>>>,
    http_client: Option<Client<HttpsConnector<HttpConnector>, Empty<Bytes>>>,

    //Socket
    socket_next_id: i32,
    tcp_sockets: HashMap<i32, TcpSocket>,
    tcp_streams: HashMap<i32, TcpStream>,
    udp_sockets: HashMap<i32, UdpSocket>,

    //Graphics
    pub graphics: Arc<Mutex<Graphics>>,
}

impl ExecutionContext {
    pub fn new(
        graphics: Arc<Mutex<Graphics>>,
        config: Config,
        log: PathBuf,
        capabilities: &[String],
    ) -> Self {
        let mut capabilities = capabilities.iter().cloned().collect::<HashSet<_>>();
        capabilities.insert("general".to_string());

        let logger = Logger::new(log);

        ExecutionContext {
            logger,

            capabilities,
            config,

            //Http
            requests: HashMap::new(),
            http_request_next_id: 0,
            prepared_requests: HashMap::new(),
            http_client: None,

            //Socket
            socket_next_id: 0,
            tcp_sockets: HashMap::new(),
            tcp_streams: HashMap::new(),
            udp_sockets: HashMap::new(),

            //Graphics
            graphics,
        }
    }

    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.contains(capability)
    }

    pub const fn logger(&mut self) -> &mut Logger {
        &mut self.logger
    }

    pub const fn config(&self) -> &Config {
        &self.config
    }

    pub fn new_http_request(&mut self) -> i32 {
        let builder = Request::builder();
        let id = self.http_request_next_id;
        self.http_request_next_id += 1;
        self.requests.insert(id, builder);
        id
    }

    pub fn get_mut_request(&mut self, id: i32) -> Option<http::request::Builder> {
        self.requests.remove(&id)
    }

    pub fn insert_request(&mut self, id: i32, builder: http::request::Builder) {
        self.requests.insert(id, builder);
    }

    pub fn insert_prepared_request(&mut self, id: i32, request: Request<Empty<Bytes>>) {
        self.prepared_requests.insert(id, request);
    }

    pub fn create_http_client(&mut self) -> i32 {
        if self.http_client.is_some() {
            error!(self.logger, "HTTP client already exists");
            return -1;
        }

        let https = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_only()
            .enable_http1()
            .build();

        self.http_client = Some(Client::builder(hyper_util::rt::TokioExecutor::new()).build(https));

        0
    }

    pub async fn send_http_request(&mut self, id: i32) -> i32 {
        let Some(http_client) = &self.http_client else {
            error!(self.logger, "HTTP client not initialized");
            return -1;
        };

        let Some(request) = self.prepared_requests.remove(&id) else {
            error!(self.logger, "HTTP request '{id}' not found");
            return -1;
        };

        match http_client.request(request).await {
            Ok(_response) => info!(self.logger, "HTTP request '{id}' completed"),
            Err(error) => {
                error!(self.logger, "HTTP request '{id}' failed: {error}");
                return -1;
            }
        }

        0
    }
}

//Sockets
impl ExecutionContext {
    pub fn new_tcp(&mut self) -> i32 {
        let socket = TcpSocket::new_v4().unwrap();
        self.tcp_sockets.insert(self.socket_next_id, socket);
        self.socket_next_id += 1;
        self.socket_next_id - 1
    }

    pub async fn tcp_connect(&mut self, id: i32, addr: SocketAddr) -> i32 {
        let Some(socket) = self.tcp_sockets.remove(&id) else {
            error!(self.logger, "Socket '{id}' not found");
            return -1;
        };

        match socket.connect(addr).await {
            Ok(stream) => {
                self.tcp_streams.insert(id, stream);
                0
            }
            Err(error) => {
                error!(
                    self.logger,
                    "Failed to connect socket '{id}' to '{addr}': {error}"
                );
                -1
            }
        }
    }

    pub async fn tcp_send(&mut self, id: i32, data: &[u8]) -> i32 {
        let Some(stream) = self.tcp_streams.get_mut(&id) else {
            error!(self.logger, "Socket '{id}' not found");
            return -1;
        };

        if let Err(error) = stream.write_all(data).await {
            error!(self.logger, "Failed to send data on socket '{id}': {error}");
            return -1;
        }

        if let Err(error) = stream.flush().await {
            error!(
                self.logger,
                "Failed to flush data on socket '{id}': {error}"
            );
            return -1;
        }

        0
    }

    pub async fn tcp_recv(&mut self, id: i32, data: &mut [u8]) -> i64 {
        let Some(stream) = self.tcp_streams.get_mut(&id) else {
            error!(self.logger, "Socket '{id}' not found");
            return -1;
        };

        match stream.read(data).await {
            Ok(n) => n as i64,
            Err(error) => {
                error!(
                    self.logger,
                    "Failed to receive data on socket '{id}': {error}"
                );
                -1
            }
        }
    }

    pub async fn tcp_shutdown(&mut self, id: i32) -> i32 {
        let Some(mut stream) = self.tcp_streams.remove(&id) else {
            error!(self.logger, "Socket '{id}' not found");
            return -1;
        };

        if let Err(error) = stream.shutdown().await {
            error!(self.logger, "Failed to shutdown socket '{id}': {error}");
            -1
        } else {
            0
        }
    }

    pub async fn new_udp(&mut self, bind_addr: SocketAddr) -> i32 {
        let socket = UdpSocket::bind(bind_addr).await.unwrap();
        self.udp_sockets.insert(self.socket_next_id, socket);
        self.socket_next_id += 1;
        self.socket_next_id - 1
    }

    pub async fn udp_connect(&mut self, id: i32, remove_addr: SocketAddr) -> i32 {
        let Some(socket) = self.udp_sockets.get(&id) else {
            error!(self.logger, "Socket '{id}' not found");
            return -1;
        };

        if let Err(error) = socket.connect(remove_addr).await {
            error!(
                self.logger,
                "Failed to connect socket '{id}' to '{remove_addr}': {error}"
            );
            return -1;
        }

        0
    }

    pub async fn udp_send(&mut self, id: i32, data: &[u8]) -> i64 {
        let Some(socket) = self.udp_sockets.get_mut(&id) else {
            error!(self.logger, "Socket '{id}' not found");
            return -1;
        };

        match socket.send(data).await {
            Ok(bytes) => bytes as i64,
            Err(error) => {
                error!(self.logger, "Failed to send data on socket '{id}': {error}");
                -1
            }
        }
    }

    pub async fn udp_recv(&mut self, id: i32, data: &mut [u8]) -> i64 {
        let Some(socket) = self.udp_sockets.get_mut(&id) else {
            error!(self.logger, "Socket '{id}' not found");
            return -1;
        };

        match socket.recv(data).await {
            Ok(bytes) => bytes as i64,
            Err(error) => {
                error!(
                    self.logger,
                    "Failed to receive data on socket '{id}': {error}"
                );
                -1
            }
        }
    }

    pub fn udp_shutdown(&mut self, id: i32) -> i32 {
        let Some(_) = self.udp_sockets.remove(&id) else {
            error!(self.logger, "Socket '{id}' not found");
            return -1;
        };

        0
    }
}
