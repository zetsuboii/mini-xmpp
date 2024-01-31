use color_eyre::eyre;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use url::Url;

pub type Stream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct Reader(SplitStream<Stream>);

impl Reader {
    pub fn from(inner: SplitStream<Stream>) -> Self {
        Self(inner)
    }

    pub async fn recv(&mut self) -> eyre::Result<String> {
        self.0
            .next()
            .await
            .and_then(|result| result.ok())
            .and_then(|message| message.into_text().ok())
            .ok_or(eyre::eyre!("no message received"))
    }
}
pub struct Writer(SplitSink<Stream, Message>);

impl Writer {
    pub fn from(inner: SplitSink<Stream, Message>) -> Self {
        Self(inner)
    }

    pub async fn send(&mut self, data: String) -> eyre::Result<()> {
        self.0.send(Message::Text(data)).await.map_err(|e| e.into())
    }
}

/// Struct to represent connection on the client side
#[derive(Debug)]
pub struct Connection {
    stream: Stream,
}

#[allow(unused)]
impl Connection {
    pub fn new(stream: Stream) -> Self {
        Self { stream }
    }

    /// Connects to the server
    pub async fn connect(url: Url) -> eyre::Result<Self> {
        let (stream, _) = tokio_tungstenite::connect_async(url).await?;
        Ok(Self::new(stream))
    }

    /// Split the stream into sink and stream
    pub fn split(self) -> (Reader, Writer) {
        let (writer_inner, reader_inner) = self.stream.split();
        (Reader::from(reader_inner), Writer::from(writer_inner))
    }

    /// Receives data from the server
    pub async fn recv(&mut self) -> eyre::Result<String> {
        self.stream
            .next()
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
