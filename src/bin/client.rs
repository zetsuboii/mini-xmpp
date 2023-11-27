use std::io::{BufRead, Write};

use color_eyre::eyre;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use mini_jabber::*;
use tokio::{io::AsyncReadExt, net::TcpStream};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

#[tokio::main]
async fn main() {
    run_client().await;
}

async fn run_client() {
    println!(":: websocket client ::");
    let address = "ws://127.0.0.1:9292";
    let url = url::Url::parse(address).expect("invalid address");

    let (ws_stream, _) = connect_async(url).await.expect("failed to connect");
    println!("websocket handshake has been successfully completed");

    let (mut writer, mut reader) = ws_stream.split();

    // Do the handshake
    handshake(&mut reader, &mut writer).await.unwrap();

    let sender = tokio::spawn(async move {
        loop {
            let mut user_input = String::new();

            // Make a new line
            print!("{}\n> ", ">".repeat(32));
            std::io::stdout().lock().flush().expect("failed to flush");

            // Read user input
            std::io::stdin()
                .lock()
                .read_line(&mut user_input)
                .expect("failed to read to string");

            // Send user input
            writer
                .send(Message::Text(user_input.trim_end().to_string()))
                .await
                .expect("failed to send message");

            let response = reader
                .get_next_text()
                .await
                .expect("failed to get response");
            println!("< {}", response);
        }
    });
    sender.await.unwrap();
}

enum HandshakeState {
    Header,
    Features,
    Done,
}

type Reader = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
type Writer = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
async fn handshake(reader: &mut Reader, writer: &mut Writer) -> color_eyre::Result<()> {
    let mut state = HandshakeState::Header;

    let initial_header = StreamHeader {
        from: "zet@mail.com".to_string(),
        to: "su@mail.com".to_string(),
        version: "1.0".to_string(),
        xml_lang: "en".to_string(),
        xmlns: "jabber:client".to_string(),
        xmlns_stream: "http://etherx.jabber.org/streams".to_string(),
    };

    loop {
        match state {
            HandshakeState::Header => {
                // Send initial header
                let serialized_header = initial_header.into_string();
                writer
                    .send(Message::Text(serialized_header))
                    .await
                    .expect("failed to send initial header");
                // Read response header
                let response_header = reader
                    .get_next_text()
                    .await
                    .expect("failed to get response");
                let response_header = StreamHeaderResponse::from_string(&response_header)
                    .expect("failed to get response");
                println!("got initial id: {}", response_header.id);

                state = HandshakeState::Features;
            }
            HandshakeState::Features => {
                let features = reader
                    .get_next_text()
                    .await
                    .expect("failed to get features");
                let features =
                    StreamFeatures::from_string(&features).expect("failed to parse features");

                // If features are empty, negotiation is over
                if features.mechanisms.is_none() && features.start_tls.is_none() {
                    state = HandshakeState::Done;
                    continue;
                }

                println!(
                    "stream mechanisms: {:?}",
                    features.mechanisms.map(|ms| {
                        ms.mechanisms
                            .into_iter()
                            .map(|m| m.0)
                            .collect::<Vec<String>>()
                    })
                );

                if let Some(tls) = features.start_tls {
                    if tls.required {
                        // Negotiate for TLS
                        let tls_feature = StartTls {
                            xmlns: "urn:ietf:params:xml:ns:xmpp-tls".to_string(),
                            required: false,
                        }
                        .into_string();
                        writer
                            .send(Message::Text(tls_feature))
                            .await
                            .expect("failed to negotiate");

                        let tls_response = reader
                            .get_next_text()
                            .await
                            .expect("failed to get response");

                        match StartTlsResponse::from_string(&tls_response) {
                            Ok(StartTlsResponse::Proceed(_)) => {}
                            Ok(StartTlsResponse::Failure(_)) => {
                                eyre::bail!("tls response failed")
                            }
                            Err(_) => {
                                eyre::bail!("failed to parse response")
                            }
                        }
                    }
                }

                state = HandshakeState::Done;
            }
            HandshakeState::Done => {
                // Reset connection by sending the header again
                let serialized_header = initial_header.into_string();
                writer
                    .send(Message::Text(serialized_header))
                    .await
                    .expect("failed to send initial header");
                // Read response header
                let response_header = reader
                    .get_next_text()
                    .await
                    .expect("failed to get response");
                let response_header = StreamHeaderResponse::from_string(&response_header)
                    .expect("failed to parse header");
                println!("new id: {}", response_header.id);
                println!("handshake done");
                return Ok(());
            }
        }
    }
}

// Our helper method which will read data from stdin and send it along the
// sender provided.
#[allow(dead_code)]
async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx.unbounded_send(Message::binary(buf)).unwrap();
    }
}
