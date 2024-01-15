use color_eyre::eyre;
use dotenvy::dotenv;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use mini_jabber::*;
use sqlx::pool::PoolConnection;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    run_server().await;
}

async fn run_server() {
    dotenv().expect(".env");

    println!(":: websocket server ::");
    let address = "127.0.0.1:9292";

    let tcp_socket = TcpListener::bind(address).await.expect("Failed to bind");
    println!("listening on {}", address);

    while let Ok((stream, _)) = tcp_socket.accept().await {
        tokio::spawn(accept_connection(stream));
    }
}

async fn accept_connection(stream: TcpStream) {
    let pool = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    println!("peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("error during the websocket handshake occurred");

    println!("new websocket connection: {}", addr);

    let (mut writer, mut reader) = ws_stream.split();

    handshake(&mut reader, &mut writer, &pool).await.unwrap();

    while let Some(raw_stanza) = reader.get_next_text().await {
        // Try to parse stanza
        let stanza = Stanza::from_string(raw_stanza.as_ref()).expect("failed to parse stanza");

        match stanza {
            Stanza::Message(message) => {
                println!(
                    "< (Message) to={} body={} [{addr}]",
                    message.to, message.body
                )
            }
            Stanza::Iq(_) => println!("< (IQ) [{addr}]"),
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

async fn handshake(
    reader: &mut Reader,
    writer: &mut Writer,
    pool: &sqlx::SqlitePool,
) -> color_eyre::Result<()> {
    let mut db_conn = pool.acquire().await?;

    reset_connection(reader, writer)
        .await
        .expect("failed to reset connection");

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
    negotiate_features(features, reader, writer)
        .await
        .expect("failed to negotitate");

    reset_connection(reader, writer)
        .await
        .expect("failed to reset connection");

    // Authenticate
    let authentication = reader
        .get_next_text()
        .await
        .expect("failed to get authentication");

    let authentication =
        Authentication::from_string(&authentication).expect("failed to parse authentication");
    let credentials = Credentials::deserialize(authentication.value);
    let valid = check_credentials(credentials, &mut db_conn)
        .await
        .expect("failed checking credentials");

    if !valid {
        eyre::bail!("failed authentication")
    }

    let success =
        AuthenticationSuccess::new("urn:ietf:params:xml:ns:xmpp-sasl".into()).into_string();
    writer
        .send(Message::Text(success))
        .await
        .expect("failed to send success message");
    println!("sent success");

    reset_connection(reader, writer)
        .await
        .expect("failed to reset connection");
    Ok(())
}

async fn check_credentials(
    credentials: Credentials,
    db_conn: &mut PoolConnection<sqlx::Sqlite>,
) -> eyre::Result<bool> {
    // Check if user exists
    let users = sqlx::query!(
        "SELECT password FROM users WHERE email = $1",
        credentials.username
    )
    .fetch_all(&mut **db_conn)
    .await?;

    if users.len() == 0 {
        sqlx::query!(
            "INSERT INTO users(email, password) VALUES($1, $2)",
            credentials.username,
            credentials.password
        )
        .execute(&mut **db_conn)
        .await?;
        Ok(true)
    } else {
        let user = &users[0];
        Ok(user.password == credentials.password)
    }
}

async fn negotiate_features(
    features: StreamFeatures,
    reader: &mut Reader,
    writer: &mut Writer,
) -> eyre::Result<()> {
    writer.send(Message::Text(features.into_string())).await?;

    // If TLS is required, negotiate TLS
    if let Some(tls) = features.start_tls {
        if tls.required {
            let next = reader
                .get_next_text()
                .await
                .ok_or(eyre::eyre!("failed to get response"))?;
            StartTls::from_string(&next)?;

            let tls_proceed = StartTlsProceed().into_string();
            writer.send(Message::Text(tls_proceed)).await?;
        }
    }

    Ok(())
}

async fn reset_connection(reader: &mut Reader, writer: &mut Writer) -> eyre::Result<()> {
    let next = reader
        .get_next_text()
        .await
        .ok_or(eyre::eyre!("failed to get header"))?;
    let stream_head = StreamHeader::from_string(&next)?;
    let stream_id = Uuid::new_v4().to_string();
    let response_head = stream_head.into_response(stream_id);

    writer
        .send(Message::Text(response_head.into_string()))
        .await?;

    Ok(())
}
