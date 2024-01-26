use color_eyre::eyre;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use parsers::jid::Jid;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

pub type Stream = WebSocketStream<TcpStream>;

/// Struct to represent connection on the server side
#[derive(Debug)]
pub struct Connection {
    /// The resource bound to this connection. It is possible to have a connection
    /// without a resource bound to it. This means that the connection is not
    /// authenticated yet.
    jid: Option<Jid>,
    /// The stream of the connection
    stream: Stream,
}

#[allow(unused)]
impl Connection {
    pub fn new(stream: Stream) -> Self {
        Self { jid: None, stream }
    }

    pub fn get_jid(&self) -> Option<&Jid> {
        self.jid.as_ref()
    }

    pub fn set_jid(&mut self, jid: Jid) {
        self.jid = Some(jid);
    }

    pub fn bound(&self) -> bool {
        self.jid.is_some()
    }

    /// Split the stream into sink and stream
    pub fn split(self) -> (SplitSink<Stream, Message>, SplitStream<Stream>) {
        self.stream.split()
    }

    /// Receives data from the server
    pub async fn read(&mut self) -> eyre::Result<String> {
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
