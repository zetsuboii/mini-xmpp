use std::io::{BufRead, Write};

use color_eyre::eyre;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use mini_jabber::*;
use tokio::{io::AsyncReadExt, net::TcpStream};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use uuid::Uuid;

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

            while user_input.ends_with("\n") {
                user_input.truncate(user_input.len() - 1);
            }

            // Send user input
            let message = Stanza::Message(StanzaMessage {
                to: "some@im.com".to_string(),
                body: user_input,
            })
            .into_string();

            writer
                .send(Message::Text(message))
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
    Authentication,
    ResourceBinding,
    Done,
}

type Reader = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
type Writer = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
async fn handshake(reader: &mut Reader, writer: &mut Writer) -> color_eyre::Result<()> {
    let mut state = HandshakeState::Header;

    loop {
        match state {
            HandshakeState::Header => {
                let conn_id = reset_connection(reader, writer)
                    .await
                    .expect("failed to start connection");
                println!("connection reset: {conn_id}");
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
                    state = HandshakeState::Authentication;
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

                state = HandshakeState::Authentication;
            }
            HandshakeState::Authentication => {
                let conn_id = reset_connection(reader, writer)
                    .await
                    .expect("failed to reset connection");
                println!("connection reset: {conn_id}");

                // Get credentials from stdin
                let mut username = String::new();
                let mut password = String::new();
                print!("username: ");
                std::io::stdout().flush().unwrap();
                std::io::stdin().read_line(&mut username).unwrap();
                username.pop();
                print!("password: ");
                std::io::stdout().flush().unwrap();
                std::io::stdin().read_line(&mut password).unwrap();
                password.pop();

                let credentials = Credentials::new(username, password);
                let authentication = Authentication::new(
                    "urn:ietf:params:xml:ns:xmpp-sasl".into(),
                    Mechanism("PLAIN".into()),
                    credentials.serialize(),
                )
                .into_string();

                writer
                    .send(Message::Text(authentication))
                    .await
                    .expect("failed to authenticate");

                let auth_response = reader
                    .get_next_text()
                    .await
                    .expect("failed to get response");

                AuthenticationSuccess::from_string(&auth_response).expect("failed to authenticate");
                state = HandshakeState::ResourceBinding;
            }
            HandshakeState::ResourceBinding => {
                let conn_id = reset_connection(reader, writer)
                    .await
                    .expect("failed to reset connection");
                println!("connection reset: {conn_id}");

                bind_resource(reader, writer)
                    .await
                    .expect("failed to bind resource");
                state = HandshakeState::Done;
            }
            HandshakeState::Done => {
                return Ok(());
            }
        }
    }
}

async fn reset_connection(reader: &mut Reader, writer: &mut Writer) -> eyre::Result<String> {
    let stream_head = StreamHeader {
        from: "zet@mail.com".to_string(),
        to: "su@mail.com".to_string(),
        version: "1.0".to_string(),
        xml_lang: "en".to_string(),
        xmlns: "jabber:client".to_string(),
        xmlns_stream: "http://etherx.jabber.org/streams".to_string(),
    };

    // Send StreamHeader to server
    writer
        .send(Message::Text(stream_head.into_string()))
        .await?;

    // Get response from the server
    let next = reader
        .get_next_text()
        .await
        .ok_or(eyre::eyre!("failed to get response"))?;

    let response_head = StreamHeaderResponse::from_string(&next)
        .map_err(|_| eyre::eyre!("failed to parse header"))?;

    Ok(response_head.id)
}

async fn bind_resource(reader: &mut Reader, writer: &mut Writer) -> eyre::Result<()> {
    // Check if bind is given inside
    let features = reader
        .get_next_text()
        .await
        .ok_or(eyre::eyre!("failed to get features"))?;
    let features = StreamFeatures::from_string(&features)?;
    features.bind.expect("bind options not found");

    // Send Iq that includes bind request, server will assign the resource
    let iq = StanzaIq {
        iq_id: Uuid::new_v4().to_string(),
        iq_type: "set".to_string(),
        iq_payload: StanzaIqPayload::Bind(IqBindPayload {
            xmlns: "urn:ietf:params:xml:ns:xmpp-bind".to_string(),
            jid: None,
            resource: None,
        }),
    };
    writer
        .send(Message::Text(Stanza::Iq(iq).into_string()))
        .await?;

    // Get the Iq that has resource assigned
    let iq_response = reader
        .get_next_text()
        .await
        .ok_or(eyre::eyre!("failed to get iq response"))?;

    let iq_response = match Stanza::from_string(&iq_response)? {
        Stanza::Iq(iq) => iq,
        _ => unreachable!("invalid iq"),
    };
    println!("new resource: {:?}", iq_response.iq_payload);

    Ok(())
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
