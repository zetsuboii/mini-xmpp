mod conn;
mod session;
mod state;

use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use conn::Connection;
use dotenvy::dotenv;
use session::Session;
use state::ServerState;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    dotenv().expect(".env");

    let address = "127.0.0.1:9292";
    let state = Arc::new(RwLock::new(ServerState::default()));
    let tcp_socket = TcpListener::bind(address).await.unwrap();

    while let Ok((stream, _)) = tcp_socket.accept().await {
        tokio::spawn(accept_connection(stream, Arc::clone(&state)));
    }
}

async fn accept_connection(stream: TcpStream, state: Arc<RwLock<ServerState>>) {
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let pool = sqlx::SqlitePool::connect(&db_url).await.unwrap();
    let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
    let conn = Connection::new(ws_stream);
    let mut session = Session::new(pool, conn);
    session.handshake().await.unwrap();

    let resource = session.get_resource().unwrap();
    let session = Arc::new(Mutex::new(session));

    // Write the session to the state
    let mut state = state.write().await;
    state.sessions.insert(resource, session.clone());

    session.lock().await.listen_for_stanzas().await;
}
