use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

/// Struct to represent connection on the client side
pub struct Connection {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl Connection {

}
