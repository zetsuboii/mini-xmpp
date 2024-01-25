use color_eyre::eyre;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use url::Url;

pub type ClientStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Struct to represent connection on the client side
#[derive(Debug)]
pub struct Connection {
    stream: ClientStream,
}

impl Connection {
    pub fn new(stream: ClientStream) -> Self {
        Self { stream }
    }

    /// Connects to the server
    pub async fn connect(url: Url) -> eyre::Result<Self> {
        let (stream, _) = tokio_tungstenite::connect_async(url).await?;
        Ok(Self::new(stream))
    }

    /// Split the stream into sink and stream
    pub fn split(self) -> (SplitSink<ClientStream, Message>, SplitStream<ClientStream>) {
        self.stream.split()
    }

    /// Receives data from the server
    pub async fn read(&mut self) -> eyre::Result<String> {
        self.stream.next()
            .await
            .ok_or(eyre::eyre!("no message received"))?
            .and_then(|message| message.into_text())
            .map_err(|e| e.into())
    }

    /// Sends data to the server
    pub async fn send(&mut self, data: String) -> eyre::Result<()> {
        self.stream
            .send(Message::Text(data))
            .await
            .map_err(|e| e.into())
    }
}
