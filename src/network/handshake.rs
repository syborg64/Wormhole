use std::net::SocketAddr;

use custom_error::custom_error;
use futures_util::{
    future::Either,
    stream::{SplitSink, SplitStream},
    SinkExt,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_tungstenite::{
    tungstenite::{self, Message},
    WebSocketStream,
};

use crate::{
    config::{GlobalConfig},
    error::WhError,
    pods::{arbo::Arbo, network::network_interface::NetworkInterface},
};

custom_error! {
    pub HandshakeError
    InvalidHandshake = "peer behaved unexpectedly",
    DuplicateHostname{hostname: String} = "peer by name {hostname} is already on this network",
    Tungstenite{source: tungstenite::Error} = "tungstenite: {source}",
    Serialization{bincode: bincode::Error} = "bincode: {bincode}",
    WhError{source: WhError} = "{source}",
    Remote{serialized: RemoteHandshakeError} = "{serialized}",
}

custom_error! {
    /// WARNING: make sure this struct is kept in sync with HandshakeError
    #[derive(Serialize, Deserialize, Clone)]
    pub RemoteHandshakeError
    InvalidHandshake = "peer behaved unexpectedly",
    DuplicateHostname{hostname: String} = "peer by name {hostname} is already on this network",
    Tungstenite{string: String} = "tungstenite: {string}",
    Serialization{string: String} = "bincode: {string}",
    WhError{string: String} = "{string}",
}

impl From<&HandshakeError> for RemoteHandshakeError {
    fn from(value: &HandshakeError) -> Self {
        type E = HandshakeError;
        type ES = RemoteHandshakeError;
        match value {
            E::InvalidHandshake => ES::InvalidHandshake,
            E::Tungstenite { source } => ES::Tungstenite {
                string: source.to_string(),
            },
            E::Serialization { bincode } => ES::Serialization {
                string: bincode.to_string(),
            },
            E::DuplicateHostname { hostname } => ES::DuplicateHostname {
                hostname: hostname.to_string(),
            },
            E::WhError { source } => ES::WhError {
                string: source.to_string(),
            },
            E::Remote { serialized } => serialized.clone(),
        }
    }
}

impl From<RemoteHandshakeError> for HandshakeError {
    fn from(value: RemoteHandshakeError) -> Self {
        type E = HandshakeError;
        type ES = RemoteHandshakeError;
        match value {
            ES::InvalidHandshake => E::InvalidHandshake,
            ES::DuplicateHostname { hostname } => E::DuplicateHostname { hostname },
            other => E::Remote {
                serialized: other.clone(),
            },
        }
    }
}

// impl Serialize for HandshakeError {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         RemoteHandshakeError::from(self).serialize(serializer)
//     }
// }

// impl<'de> Deserialize<'de> for HandshakeError {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         Ok(RemoteHandshakeError::deserialize(deserializer)?.into())
//     }
// }

impl From<bincode::Error> for HandshakeError {
    fn from(bincode: bincode::Error) -> Self {
        HandshakeError::Serialization { bincode }
    }
}

const GIT_HASH: &'static str = env!("GIT_HASH");

#[derive(Deserialize, Serialize)]
enum Handshake {
    Accept(Accept),
    Connect(Connect),
    Refuse(RemoteHandshakeError),
    Wave(Wave),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Accept {
    pub hostname: String,
    pub config: GlobalConfig,
    pub arbo: Arbo,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Connect {
    pub magic_version: String,
    pub hostname: String,
    pub socket: Option<SocketAddr>,
    // pub authentification: String,
    // pub files: HashMap<WhPath, Metadata>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Wave {
    pub hostname: String,
    pub blame: String,
}

pub async fn accept<T: StreamExt<Item = Result<Message, tungstenite::Error>> + SinkExt<Message>>(
    stream: &mut SplitStream<WebSocketStream<TcpStream>>,
    sink: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    network: NetworkInterface,
) -> Result<Either<Connect, Wave>, HandshakeError> {
    let message = stream
        .next()
        .await
        .map(|res| res.map_err(Into::into))
        .unwrap_or(Err(HandshakeError::InvalidHandshake));
    let handshake = match message {
        Ok(Message::Binary(bytes)) => bincode::deserialize::<Handshake>(&bytes).map_err(From::from),
        Ok(_) => Err(HandshakeError::InvalidHandshake),
        Err(e) => Err(e.into()),
    };

    let result = match handshake {
        Ok(Handshake::Connect(connect)) => {
            (async || {
                // closures to capture ? process
                let peers_lock = network.peers.read();

                peers_lock
                    .iter()
                    .all(|peer| peer.hostname != connect.hostname)
                    .then_some(())
                    .ok_or(HandshakeError::DuplicateHostname {
                        hostname: connect.hostname.clone(),
                    })?;
                let accept = Accept {
                    hostname: network.hostname()?,
                    config: network.global_config.read().clone(),
                    arbo: (*network.arbo.read()).clone(),
                };
                let data = bincode::serialize(&accept)?;
                sink.send(Message::Binary(data.into())).await?;

                Ok(Either::Left(connect))
            })()
            .await
        }
        Ok(Handshake::Wave(wave)) => {
            (async || {
                // closures to capture ? process
                let wave_back = Wave {
                    hostname: network.hostname()?,
                    blame: wave.hostname.clone(),
                };
                let data = bincode::serialize(&Handshake::Wave(wave_back))?;
                sink.send(Message::Binary(data.into())).await?;

                Ok(Either::Right(wave))
            })()
            .await
        }
        Ok(_) => Err(HandshakeError::InvalidHandshake),
        Err(e) => Err(e),
    };
    let result = if let Err(error) = result {
        (async ||
            { sink.send(Message::Binary(bincode::serialize(&Handshake::Refuse(
                (&error).into(),
            ))?.into()))
            .await?;
            Err(error)
        }
        )()
        .await
    } else {
        result
    };
    result
}

pub async fn wave(
    stream: &mut SplitStream<WebSocketStream<TcpStream>>,
    sink: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    hostname: String,
    blame: String,
) -> Result<Wave, HandshakeError> {
    let wave = Wave {
        hostname,
        blame,
    };

    let serialized = bincode::serialize(&Handshake::Wave(wave))?;
    sink.send(Message::Binary(serialized.into())).await?;


    let response = stream.next().await.ok_or(HandshakeError::InvalidHandshake)??;

    let handshake = if let Message::Binary(bytes) = response {
        Ok(bincode::deserialize::<Handshake>(&bytes)?)
    } else {
        Err(HandshakeError::InvalidHandshake)
    }?;

    match handshake {
        Handshake::Wave(wave) => Ok(wave),
        _ => Err(HandshakeError::InvalidHandshake)
    }
}

pub async fn connect(
    stream: &mut SplitStream<WebSocketStream<TcpStream>>,
    sink: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    hostname: String,
) -> Result<Accept, HandshakeError> {
    let connect = Connect {
        hostname,
        magic_version: GIT_HASH.into(),
        socket: None,
    };

    let serialized = bincode::serialize(&Handshake::Connect(connect))?;
    sink.send(Message::Binary(serialized.into())).await?;


    let response = stream.next().await.ok_or(HandshakeError::InvalidHandshake)??;

    let handshake = if let Message::Binary(bytes) = response {
        Ok(bincode::deserialize::<Handshake>(&bytes)?)
    } else {
        Err(HandshakeError::InvalidHandshake)
    }?;

    match handshake {
        Handshake::Accept(accept) => Ok(accept),
        Handshake::Refuse(error) => Err(error.into()),
        _ => Err(HandshakeError::InvalidHandshake)
    }
}
