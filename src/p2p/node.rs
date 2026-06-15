use std::collections::HashMap;
use std::sync::Arc;

use iroh::EndpointId;
use n0_future::boxed::BoxStream;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::sync::broadcast;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error(transparent)]
    Bind(#[from] iroh::endpoint::BindError),
    #[error(transparent)]
    ClosedStream(#[from] iroh::endpoint::ClosedStream),
    #[error(transparent)]
    Connection(#[from] iroh::endpoint::ConnectionError),
    #[error(transparent)]
    Connect(#[from] iroh::endpoint::ConnectError),
    #[error(transparent)]
    AcceptProtocol(#[from] iroh::protocol::AcceptError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum NodeMessages {
    Hello {
        endpoint_id: EndpointId,
        name: String,
    },
    Welcome {
        endpoint_id: EndpointId,
        name: String,
    },
    NameUpdate {
        endpoint_id: EndpointId,
        old_name: String,
        new_name: String,
    },
    Accepted {
        endpoint_id: EndpointId,
    },
    Closed {
        endpoint_id: EndpointId,
        error: Option<String>,
    },
    CreateNote {
        id: String,
        content: String,
        category: String,
        author: String,
    },
    VoteToggle {
        note_id: String,
        peer_id: EndpointId,
    },
}

#[derive(Clone)]
pub(crate) struct RetroNode {
    router: iroh::protocol::Router,
    accept_events: broadcast::Sender<NodeMessages>,
    peers: Arc<Mutex<HashMap<EndpointId, iroh::endpoint::Connection>>>,
    pub endpoint_id: EndpointId,
}

impl RetroNode {
    pub(crate) async fn host(sk: iroh::SecretKey) -> Result<Self, Error> {
        let endpoint = iroh::Endpoint::builder(iroh::endpoint::presets::N0)
            .secret_key(sk.clone())
            .alpns(vec![crate::ALPN.to_vec()])
            .bind()
            .await?;

        let endpoint_id = sk.public();
        let (event_sender, _event_receiver) = broadcast::channel(256);
        let peers = Arc::new(Mutex::new(HashMap::new()));

        let handler = RetroHandler::new(event_sender.clone(), peers.clone());
        let router = iroh::protocol::Router::builder(endpoint.clone())
            .accept(crate::ALPN, handler)
            .spawn();

        Ok(Self {
            router,
            accept_events: event_sender,
            peers,
            endpoint_id,
        })
    }

    pub(crate) async fn connect(pk: iroh::PublicKey) -> Result<Self, Error> {
        let client_sk = iroh::SecretKey::generate();
        let endpoint = iroh::Endpoint::builder(iroh::endpoint::presets::N0)
            .secret_key(client_sk.clone())
            .alpns(vec![crate::ALPN.to_vec()])
            .bind()
            .await?;

        let endpoint_id = client_sk.public();
        let addr = iroh::EndpointAddr::from(pk);
        let connection = endpoint.connect(addr, crate::ALPN).await?;

        let (event_sender, _) = broadcast::channel(256);
        let peers = Arc::new(Mutex::new(HashMap::new()));

        peers.lock().await.insert(pk.into(), connection.clone());

        let handler = RetroHandler::new(event_sender.clone(), peers.clone());
        let router = iroh::protocol::Router::builder(endpoint.clone())
            .accept(crate::ALPN, handler)
            .spawn();

        let event_sender_clone = event_sender.clone();
        let conn_clone = connection.clone();
        let peers_clone = peers.clone();
        leptos::task::spawn_local(async move {
            let _ = RetroHandler::listen_to_connection(conn_clone, event_sender_clone, peers_clone)
                .await;
        });

        Ok(Self {
            router,
            accept_events: event_sender,
            peers,
            endpoint_id,
        })
    }

    pub fn accept_events(&self) -> BoxStream<NodeMessages> {
        let receiver = self.accept_events.subscribe();
        Box::pin(BroadcastStream::new(receiver).filter_map(|event| event.ok()))
    }

    pub async fn broadcast(&self, msg: &NodeMessages) {
        let peers = self.peers.lock().await;
        let payload = serde_json::to_vec(msg).unwrap();

        tracing::info!("Sending broadcast message to {} peers", peers.len());
        for (id, conn) in peers.iter() {
            if let Ok((mut send, _recv)) = conn.open_bi().await {
                if let Ok(_) = send.write_all(&payload).await {
                    let _ = send.finish();
                    tracing::info!("Message transmitted to peer {id}");
                }
            } else {
                tracing::error!("Failed to open bi-directional stream to peer {id}");
            }
        }
    }

    pub async fn close(self) {
        let _ = self.router.shutdown().await;
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RetroHandler {
    event_sender: broadcast::Sender<NodeMessages>,
    peers: Arc<Mutex<HashMap<EndpointId, iroh::endpoint::Connection>>>,
}

impl RetroHandler {
    pub(crate) fn new(
        event_sender: broadcast::Sender<NodeMessages>,
        peers: Arc<Mutex<HashMap<EndpointId, iroh::endpoint::Connection>>>,
    ) -> Self {
        Self {
            event_sender,
            peers,
        }
    }

    async fn handle_connection(
        self,
        connection: iroh::endpoint::Connection,
    ) -> std::result::Result<(), iroh::protocol::AcceptError> {
        let endpoint_id = connection.remote_id();

        self.peers
            .lock()
            .await
            .insert(endpoint_id, connection.clone());

        let _ = self
            .event_sender
            .send(NodeMessages::Accepted { endpoint_id });

        let res =
            Self::listen_to_connection(connection, self.event_sender.clone(), self.peers.clone())
                .await;

        self.peers.lock().await.remove(&endpoint_id);
        let error = res.as_ref().err().map(|err| err.to_string());
        let _ = self
            .event_sender
            .send(NodeMessages::Closed { endpoint_id, error });
        res
    }

    pub async fn listen_to_connection(
        connection: iroh::endpoint::Connection,
        event_sender: broadcast::Sender<NodeMessages>,
        peers: Arc<Mutex<HashMap<EndpointId, iroh::endpoint::Connection>>>,
    ) -> std::result::Result<(), iroh::protocol::AcceptError> {
        let sender_id = connection.remote_id();

        loop {
            let (mut send, mut recv) = connection.accept_bi().await?;
            let buffer = recv
                .read_to_end(1024 * 64)
                .await
                .map_err(|e| iroh::protocol::AcceptError::from_err(e))?;

            if buffer.is_empty() {
                break;
            }

            if let Ok(msg) = serde_json::from_slice::<NodeMessages>(&buffer) {
                tracing::info!("received message from {}: {:?}", sender_id, msg);
                let _ = event_sender.send(msg.clone());

                let peers_lock = peers.lock().await;
                if peers_lock.len() > 1 {
                    let payload = serde_json::to_vec(&msg).unwrap();
                    for (id, conn) in peers_lock.iter() {
                        if *id != sender_id {
                            if let Ok((mut s_send, _)) = conn.open_bi().await {
                                if let Ok(_) = s_send.write_all(&payload).await {
                                    let _ = s_send.finish();
                                }
                            }
                        }
                    }
                }
            }
            let _ = send.finish();
        }
        Ok(())
    }
}

impl iroh::protocol::ProtocolHandler for RetroHandler {
    async fn accept(
        &self,
        connection: iroh::endpoint::Connection,
    ) -> Result<(), iroh::protocol::AcceptError> {
        self.clone().handle_connection(connection).await
    }
}
