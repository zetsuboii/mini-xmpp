use std::{collections::HashMap, rc::Rc, sync::Arc};

use color_eyre::eyre;
use dotenvy::dotenv;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use mini_jabber::*;
use sqlx::pool::PoolConnection;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Mutex, RwLock},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use uuid::Uuid;

type Reader = SplitStream<WebSocketStream<TcpStream>>;
type Writer = SplitSink<WebSocketStream<TcpStream>, Message>;

#[derive(Debug)]
struct ClientConnection {
    resource: String,
    reader: Arc<Mutex<Reader>>,
    writer: Arc<Mutex<Writer>>,
}

impl ClientConnection {
    fn new(resource: String, reader: Arc<Mutex<Reader>>, writer: Arc<Mutex<Writer>>) -> Self {
        Self {
            resource,
            reader,
            writer,
        }
    }

    fn resource(&self) -> &str {
        self.resource.as_ref()
    }

    fn reader(&self) -> &Mutex<Reader> {
        &self.reader
    }

    fn writer(&self) -> &Mutex<Writer> {
        &self.writer
    }
}

#[derive(Default, Debug)]
struct ServerState {
    connected_clients: HashMap<String, Vec<ClientConnection>>,
}

#[tokio::main]
async fn main() {
    run_server().await;
}

async fn run_server() {
    dotenv().expect(".env");

    println!(":: websocket server ::");
    let address = "127.0.0.1:9292";

    let state = Arc::new(RwLock::new(ServerState::default()));

    let tcp_socket = TcpListener::bind(address).await.expect("Failed to bind");
    println!("listening on {}", address);

    while let Ok((stream, _)) = tcp_socket.accept().await {
        tokio::spawn(accept_connection(stream, Arc::clone(&state)));
    }
}

async fn accept_connection(stream: TcpStream, state: Arc<RwLock<ServerState>>) {
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

    let (writer, reader) = ws_stream.split();
    let writer = Arc::new(Mutex::new(writer));
    let reader = Arc::new(Mutex::new(reader));

    let jid = handshake(&Arc::clone(&reader), &Arc::clone(&writer), &pool)
        .await
        .unwrap();

    // Save client to the state
    let mut state = state.write().await;

    let conn_key = jid.address();
    let conn_val = (*state).connected_clients.get(&conn_key);
    if conn_val.is_none() {
        (*state)
            .connected_clients
            .insert(conn_key.clone(), Vec::new());
    }
    if let Some(conns) = (*state).connected_clients.get_mut(&conn_key) {
        conns.push(ClientConnection::new(jid.resource_part, Arc::clone(&reader), Arc::clone(&writer)));
    }
    println!("{:?}", &state);
    drop(state);

    while let Some(raw_stanza) = reader.lock().await.get_next_text().await {
        // Try to parse stanza
        let stanza = Stanza::try_from(raw_stanza.as_ref()).expect("failed to parse stanza");

        match stanza {
            Stanza::Message(message) => {
                println!("< {:?} [{addr}]", message)
            }
            Stanza::Iq(_) => println!("< (IQ) [{addr}]"),
            Stanza::Presence => println!("< (Presence) [{addr}]"),
        }

        writer
            .lock()
            .await
            .send(Message::Text("ack".to_string()))
            .await
            .expect("failed to send ack");

        println!("> ack");
    }
}

async fn handshake(
    reader: &Mutex<Reader>,
    writer: &Mutex<Writer>,
    pool: &sqlx::SqlitePool,
) -> eyre::Result<Jid> {
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
        bind: None,
    };
    negotiate_features(features, reader, writer)
        .await
        .expect("failed to negotitate");

    reset_connection(reader, writer)
        .await
        .expect("failed to reset connection");

    // Authenticate
    let authentication = reader
        .lock()
        .await
        .get_next_text()
        .await
        .expect("failed to get authentication");

    let authentication =
        Authentication::try_from(authentication.as_ref()).expect("failed to parse authentication");
    let credentials = Credentials::from_base64(authentication.value);
    let valid = check_credentials(&credentials, &mut db_conn)
        .await
        .expect("failed checking credentials");
    if !valid {
        eyre::bail!("failed authentication")
    }
    let jid = credentials.username;
    let (local_part, domain_part) = jid.split_at(jid.find("@").expect("invalid jid"));

    let success = AuthenticationSuccess::new("urn:ietf:params:xml:ns:xmpp-sasl".into()).to_string();
    writer
        .lock()
        .await
        .send(Message::Text(success))
        .await
        .expect("failed to send success message");

    reset_connection(reader, writer)
        .await
        .expect("failed to reset connection");

    // After authentication, server has to send bind feature
    let stream_features = StreamFeatures {
        bind: Some(Bind {
            xmlns: "urn:ietf:params:xml:ns:xmpp-bind".to_string(),
        }),
        mechanisms: None,
        start_tls: None,
    };
    negotiate_features(stream_features, reader, writer)
        .await
        .expect("failed to negotiate");

    // After sending features, client will ask for a resource and server has to
    // generate it
    let jid = generate_jid(
        reader,
        writer,
        local_part.to_string(),
        domain_part[1..].to_string(),
    )
    .await
    .expect("failed to generate resource");

    Ok(jid)
}

async fn check_credentials(
    credentials: &Credentials,
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
    reader: &Mutex<Reader>,
    writer: &Mutex<Writer>,
) -> eyre::Result<()> {
    let mut reader = reader.lock().await;
    let mut writer = writer.lock().await;

    writer.send(Message::Text(features.to_string())).await?;

    // If TLS is required, negotiate TLS
    if let Some(tls) = features.start_tls {
        if tls.required {
            let next = reader
                .get_next_text()
                .await
                .ok_or(eyre::eyre!("failed to get response"))?;
            StartTls::try_from(next.as_ref())?;

            let tls_proceed = StartTlsProceed().to_string();
            writer.send(Message::Text(tls_proceed)).await?;
        }
    }

    Ok(())
}

async fn generate_jid(
    reader: &Mutex<Reader>,
    writer: &Mutex<Writer>,
    local_part: String,
    domain_part: String,
) -> eyre::Result<Jid> {
    let mut reader = reader.lock().await;
    let mut writer = writer.lock().await;

    let next = reader
        .get_next_text()
        .await
        .ok_or(eyre::eyre!("failed to get bind request"))?;

    let bind_request = match Stanza::try_from(next.as_ref())? {
        Stanza::Iq(iq) => iq,
        _ => eyre::bail!("invalid bind request"),
    };

    let resource_part = Uuid::new_v4().to_string();
    let jid = Jid::new(local_part, domain_part, resource_part);
    let bind_response = Stanza::Iq(StanzaIq {
        id: bind_request.id,
        type_: "result".to_string(),
        payload: StanzaIqPayload::Bind(IqBindPayload {
            xmlns: "urn:ietf:params:xml:ns:xmpp-bind".to_string(),
            jid: Some(jid.to_string()),
            resource: None,
        }),
    });

    writer
        .send(Message::Text(bind_response.to_string()))
        .await?;

    Ok(jid)
}

async fn reset_connection(reader: &Mutex<Reader>, writer: &Mutex<Writer>) -> eyre::Result<()> {
    let mut reader = reader.lock().await;
    let mut writer = writer.lock().await;

    let next = reader
        .get_next_text()
        .await
        .ok_or(eyre::eyre!("failed to get header"))?;
    let stream_head = StreamHeader::try_from(next.as_ref())?;
    let stream_id = Uuid::new_v4().to_string();
    let response_head = stream_head.into_response(stream_id);

    writer
        .send(Message::Text(response_head.to_string()))
        .await?;

    Ok(())
}
