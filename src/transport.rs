use std::collections::{HashMap, hash_map::Entry::*};

use std::net::SocketAddr;

use tracing::{error, info};

use std::sync::Arc;
use tokio::sync::Mutex;

use tonic::transport::Server;
use rand::seq::SliceRandom;
use crate::error::Error;

/// event protos
pub mod events {
    tonic::include_proto!("events");
}

pub use events::Event;
pub use events::events_server::{Events, EventsServer};
use events::events_client::*;

async fn connect_client(port: &u32) -> Result<EventsClient<tonic::transport::Channel>, Error> {
    EventsClient::connect(format!("http://[::1]:{}", port))
        .await.map_err(|e| Error::TransportError {
        message: format!("Failed to connect to client: {}", port),
        cause: e,
    })
}

#[async_trait::async_trait]
pub trait IncomingHandler: Send + Sync + 'static {
    async fn handle_event(&self, event: Event);
}

struct CompositeHandler {
    handlers: Vec<Box<dyn IncomingHandler>>,
}

#[async_trait::async_trait]
impl IncomingHandler for CompositeHandler {
    async fn handle_event(&self, event: Event) {
        for handler in &self.handlers[..] {
            handler.handle_event(event.clone()).await;
        }
    }
}

pub struct EventsServerBuilder {
    handler: CompositeHandler,
}

impl EventsServerBuilder {
    pub fn new() -> Self {
        EventsServerBuilder {
            handler: CompositeHandler { handlers: vec![] },
        }
    }

    pub fn add(mut self, handler: Box<dyn IncomingHandler>) -> Self {
        self.handler.handlers.push(handler);

        return self;
    }

    pub fn redirecting(mut self, targets: &Vec<u32>) -> Self {
        let handler = RedirectHandler {
            targets: targets.clone(),
            pool: ClientPool::new(),
        };

        self.handler.handlers.push(Box::new(handler));

        return self;
    }


    pub async fn run(self, addr: SocketAddr) -> Result<(), String> {
        let inner = EventServer {
            handler: Arc::new(self.handler),
        };

        let server = EventsServer::new(inner);

        Server::builder()
            .add_service(server)
            .serve(addr).await.map_err(|e| e.to_string())
    }
}

pub struct EventServer {
    handler: Arc<dyn IncomingHandler>,
}

#[async_trait::async_trait]
impl Events for EventServer {
    async fn notify(
        &self,
        request: tonic::Request<Event>,
    ) -> Result<tonic::Response<events::Void>, tonic::Status> {
        let event = request.into_inner();

        let this_handler = self.handler.clone();
        tokio::spawn(async move {
            this_handler.handle_event(event).await;
        });

        Ok(tonic::Response::new(events::Void {}))
    }
}

struct RedirectHandler {
    targets: Vec<u32>,
    pool: ClientPool,
}

#[async_trait::async_trait]
impl IncomingHandler for RedirectHandler {
    async fn handle_event(&self, event: Event) {
        let target = self.targets.choose(&mut rand::thread_rng());

        if target.is_none() {
            error!(targets = format!("{:?}", self.targets).as_str(),
                   "No targets specified");
            return;
        }

        let target_port = target.unwrap();

        let wrapped_client = self.pool.select_client_for(target_port.clone()).await;
        if let Err(e) = wrapped_client {
            error!(target = target_port,
                   error = format!("{:?}", e).as_str(),
                   "Unable to connect to target");
            return;
        }

        let protected_client = wrapped_client.unwrap();
        let mut client = protected_client.lock().await;

        let response = client.notify(tonic::Request::new(event)).await;

        match response {
            Ok(_) => info!(target = target_port, "Target received event"),
            Err(e) => error!(target = target_port,
                             code = e.code() as i32,
                             message = e.message(),
                             "Target responded with non-OK status"),
        };
    }
}

struct ClientPool {
    clients: Mutex<HashMap<u32, Arc<Mutex<EventsClient<tonic::transport::Channel>>>>>,
}

impl ClientPool {
    fn new() -> Self {
        ClientPool {
            clients: Mutex::new(HashMap::new()),
        }
    }

    async fn select_client_for(&self, port: u32) -> Result<Arc<Mutex<EventsClient<tonic::transport::Channel>>>, Error> {
        let mut guard = self.clients.lock().await;

        let val = match guard.entry(port) {
            Vacant(entry) => {
                let client = connect_client(&port).await?;

                entry.insert(Arc::new(Mutex::new(client))).clone()
            }
            Occupied(entry) => entry.get().clone(),
        };
        Ok(val)
    }
}

#[cfg(test)]
mod transport_tests {
    use super::*;

    const PORT: u32 = 8080;

    #[tokio::test]
    async fn test_transport() {
        let event = Event {
            name: "some event".to_string(),
        };

        let mut client = connect_client(&PORT).await.unwrap();

        let _ = client.notify(tonic::Request::new(event.clone())).await.unwrap();
    }
}
