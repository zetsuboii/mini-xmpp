use std::io::{BufRead, Write};

use color_eyre::eyre;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use mini_jabber::*;
use quick_xml::escape::unescape;
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
    let (mut writer, mut reader) = ws_stream.split();

    // Do the handshake
    let jid = handshake(&mut reader, &mut writer).await.unwrap();

    let receiver = tokio::spawn(async move {
        loop {
            let response = reader
                .get_next_text()
                .await
                .map(|text| Stanza::try_from(text.as_ref()).ok())
                .flatten()
                .expect("failed to get response");
            let response = match response {
                Stanza::Message(message) => message,
                _ => return,
            };

            let from = response.from.unwrap_or("unknown".to_string());
            let body = response.body.unwrap_or("".to_string());

            println!("\rfrom: {}", from);
            println!("< {}", unescape(body.as_ref()).unwrap());
            print!("{}\nto: ", "=".repeat(32));
            std::io::stdout().lock().flush().expect("failed to flush");
        }
    });

    let sender = tokio::spawn(async move {
        loop {
            // Make a new line
            print!("{}\nto: ", "=".repeat(32));
            std::io::stdout().lock().flush().expect("failed to flush");
            let to = get_line();

            // Make a new line
            print!("> ");
            std::io::stdout().lock().flush().expect("failed to flush");
            let input = get_line();

            // Send user input
            let message = Stanza::Message(StanzaMessage {
                id: None,
                from: Some(jid.clone()),
                to: Some(to),
                body: Some(input),
                xml_lang: Some("en".to_string()),
            })
            .to_string();

            writer
                .send(Message::Text(message))
                .await
                .expect("failed to send message");
        }
    });

    receiver.await.unwrap();
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
async fn handshake(reader: &mut Reader, writer: &mut Writer) -> eyre::Result<String> {
    let mut state = HandshakeState::Header;
    let mut jid: Option<String> = None;

    loop {
        match state {
            HandshakeState::Header => {
                reset_connection(reader, writer)
                    .await
                    .expect("failed to start connection");
                state = HandshakeState::Features;
            }
            HandshakeState::Features => {
                let features = reader
                    .get_next_text()
                    .await
                    .expect("failed to get features");
                let features =
                    StreamFeatures::try_from(features.as_ref()).expect("failed to parse features");

                // If features are empty, negotiation is over
                if features.mechanisms.is_none() && features.start_tls.is_none() {
                    state = HandshakeState::Authentication;
                    continue;
                }

                if let Some(tls) = features.start_tls {
                    if tls.required {
                        // Negotiate for TLS
                        let tls_feature = StartTls {
                            xmlns: "urn:ietf:params:xml:ns:xmpp-tls".to_string(),
                            required: false,
                        }
                        .to_string();
                        writer
                            .send(Message::Text(tls_feature))
                            .await
                            .expect("failed to negotiate");

                        let tls_response = reader
                            .get_next_text()
                            .await
                            .expect("failed to get response");

                        match StartTlsResponse::try_from(tls_response.as_ref()) {
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
                reset_connection(reader, writer)
                    .await
                    .expect("failed to reset connection");

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
                    credentials.to_base64(),
                )
                .to_string();

                writer
                    .send(Message::Text(authentication))
                    .await
                    .expect("failed to authenticate");

                let auth_response = reader
                    .get_next_text()
                    .await
                    .expect("failed to get response");

                AuthenticationSuccess::try_from(auth_response.as_ref())
                    .expect("failed to authenticate");
                state = HandshakeState::ResourceBinding;
            }
            HandshakeState::ResourceBinding => {
                reset_connection(reader, writer)
                    .await
                    .expect("failed to reset connection");

                let bind_response = bind_resource(reader, writer)
                    .await
                    .expect("failed to bind resource");
                jid = Some(bind_response);

                state = HandshakeState::Done;
            }
            HandshakeState::Done => {
                return Ok(jid.expect("jid not found"));
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
    writer.send(Message::Text(stream_head.to_string())).await?;

    // Get response from the server
    let next = reader
        .get_next_text()
        .await
        .ok_or(eyre::eyre!("failed to get response"))?;

    let response_head = StreamHeaderResponse::try_from(next.as_ref())
        .map_err(|_| eyre::eyre!("failed to parse header"))?;

    Ok(response_head.id)
}

async fn bind_resource(reader: &mut Reader, writer: &mut Writer) -> eyre::Result<String> {
    // Check if bind is given inside
    let features = reader
        .get_next_text()
        .await
        .ok_or(eyre::eyre!("failed to get features"))?;
    let features = StreamFeatures::try_from(features.as_ref())?;
    features.bind.expect("bind options not found");

    // Send Iq that includes bind request, server will assign the resource
    let iq = StanzaIq {
        id: Uuid::new_v4().to_string(),
        type_: "set".to_string(),
        payload: StanzaIqPayload::Bind(IqBindPayload {
            xmlns: "urn:ietf:params:xml:ns:xmpp-bind".to_string(),
            jid: None,
            resource: None,
        }),
    };
    writer
        .send(Message::Text(Stanza::Iq(iq).to_string()))
        .await?;

    // Get the Iq that has resource assigned
    let iq_response = reader
        .get_next_text()
        .await
        .ok_or(eyre::eyre!("failed to get iq response"))?;

    let iq_response = match Stanza::try_from(iq_response.as_ref())? {
        Stanza::Iq(iq) => iq,
        _ => unreachable!("invalid iq"),
    };

    let iq_payload = match iq_response.payload {
        StanzaIqPayload::Bind(payload) => payload,
    };

    iq_payload.jid.ok_or(eyre::eyre!("jid not found"))
}

fn get_line() -> String {
    let mut input = String::new();

    // Read user input
    std::io::stdin()
        .lock()
        .read_line(&mut input)
        .expect("failed to read to string");

    while input.ends_with("\n") {
        input.truncate(input.len() - 1);
    }

    input
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
