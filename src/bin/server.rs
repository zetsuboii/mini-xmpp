use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use mini_jabber::*;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

#[tokio::main]
async fn main() {
    run_server().await;
}

async fn run_server() {
    println!(":: websocket server ::");
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

    let (mut writer, mut reader) = ws_stream.split();

    handshake(&mut reader, &mut writer).await.unwrap();

    while let Some(raw_stanza) = reader.get_next_text().await {
        // Try to parse stanza
        let stanza = Stanza::from_string(raw_stanza.as_ref()).expect("failed to parse stanza");

        match stanza {
            Stanza::Message(message) => {
                println!("< (Message) to={} body={} [{addr}]", message.to, message.body)
            }
            Stanza::Iq => println!("< (IQ) [{addr}]"),
            Stanza::Presence => println!("< (Presence) [{addr}]"),
        }

        writer
            .send(Message::Text("ack".to_string()))
            .await
            .expect("failed to send ack");

        println!("> ack");
    }
}

type Reader = SplitStream<WebSocketStream<TcpStream>>;
type Writer = SplitSink<WebSocketStream<TcpStream>, Message>;

async fn handshake(reader: &mut Reader, writer: &mut Writer) -> color_eyre::Result<()> {
    // Read initial header
    let initial_header = reader.get_next_text().await.expect("failed to get header");
    let initial_header =
        StreamHeader::from_string(&initial_header).expect("failed to parse header");

    // Append id to header
    let id = "++123456789++".to_string();
    let mut response_header = initial_header.into_response(id);
    response_header.xmlns = "jabber:server".to_string();
    let response_header = response_header.into_string();

    // Send response header
    writer
        .send(Message::Text(response_header))
        .await
        .expect("failed to send hello message");

    // Send features header
    let features = StreamFeatures {
        mechanisms: Some(Mechanisms {
            xmlns: "urn:ietf:params:xml:ns:xmpp-sasl".to_string(),
            mechanisms: vec![Mechanism("PLAIN".into())],
        }),
        start_tls: Some(StartTls {
            xmlns: "urn:ietf:params:xml:ns:xmpp-tls".to_string(),
            required: true,
        }),
    };
    let features = features.into_string();
    writer
        .send(Message::Text(features))
        .await
        .expect("failed to send features");

    // Get features back and send proceed message
    let tls_response = reader
        .get_next_text()
        .await
        .expect("failed to get tls response");
    StartTls::from_string(&tls_response).expect("failed to parse tls repsonse");

    let tls_proceed = StartTlsProceed().into_string();
    writer
        .send(Message::Text(tls_proceed))
        .await
        .expect("failed to send proceed message");

    // Start connection again
    let initial_header = reader.get_next_text().await.expect("failed to get header");
    let initial_header =
        StreamHeader::from_string(&initial_header).expect("failed to parse header");
    let id = "++98765321++".to_string();
    let mut response_header = initial_header.into_response(id);
    response_header.xmlns = "jabber:server".to_string();
    let response_header = response_header.into_string();
    writer
        .send(Message::Text(response_header))
        .await
        .expect("failed to send hello message");

    println!("handshake done");
    Ok(())
}
