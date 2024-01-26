use parsers::{
    jid::Jid,
    stanza::{message, Stanza},
    stream::auth::PlaintextCredentials,
};

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

    let username = get_user_input("Enter username:");
    let password = get_user_input("Enter password:");

    let jid = Jid::try_from(username.clone()).unwrap();
    let credentials = PlaintextCredentials::new(username, password);

    let conn = Connection::connect(url).await.unwrap();
    let mut session = Session::new(jid.clone(), credentials, conn);

    session.handshake().await.unwrap();
    println!("Handshake successful");

    let stanza = Stanza::Message(message::Message {
        id: None,
        from: Some(jid.to_string()),
        to: Some("other@mail".into()),
        body: Some("Hello world".into()),
        xml_lang: Some("en".into()),
    });
    session.send_stanza(stanza).await.unwrap();
}
