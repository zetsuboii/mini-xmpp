use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() {
    echo_server().await;
}

async fn echo_server() {
    println!(":: websocket echo server ::");
    let address = "127.0.0.1:9292";

    let tcp_socket = TcpListener::bind(address).await.expect("Failed to bind");
    println!("listening on {}", address);

    while let Ok((stream, _)) = tcp_socket.accept().await {
        tokio::spawn(accept_connection(stream));
    }
}

async fn accept_connection(stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    println!("peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("error during the websocket handshake occurred");

    println!("new websocket connection: {}", addr);

    let (mut write, read) = ws_stream.split();

    // Send hello message
    write
        .send(Message::Text("hello\n".into()))
        .await
        .expect("failed to send hello message");

    // Forward what's read to write
    read.forward(write).await.expect("forwarding failed");
}
