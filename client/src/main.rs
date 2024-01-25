use parsers::{jid::Jid, stream::auth::PlaintextCredentials};

use crate::{conn::Connection, session::Session};

mod conn;
mod session;

fn get_user_input(prompt: &'static str) -> String {
    let mut input = String::new();
    println!("{}", prompt);
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

#[tokio::main]
async fn main() {
    let address = "ws://127.0.0.1:9292";
    let url = url::Url::parse(address).expect("invalid address");

    let jid = get_user_input("Enter JID:");
    let username = get_user_input("Enter username:");
    let password = get_user_input("Enter password:");

    let jid = Jid::try_from(jid).unwrap();
    let credentials = PlaintextCredentials::new(username, password);

    let conn = Connection::connect(url).await.unwrap();
    let mut session = Session::new(jid, credentials, conn);

    session.handshake().await.unwrap();
    println!("Handshake successful");
}
