use parsers::{
    constants::NAMESPACE_FRIENDS,
    jid::Jid,
    stanza::{iq, presence, Stanza},
    stream::auth::PlaintextCredentials,
};
use uuid::Uuid;

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
    println!(":: xmpp client ::");
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

    // Send presence message
    let presence = Stanza::Presence(presence::Presence {
        id: Uuid::new_v4().to_string().into(),
        from: jid.to_string().into(),
        ..Default::default()
    });
    session.send_stanza(presence).await.unwrap();

    // Get connected clients
    let friends_iq = Stanza::Iq(iq::Iq {
        id: Uuid::new_v4().to_string(),
        from: jid.to_string().into(),
        type_: "get".to_string().into(),
        payload: iq::Payload::Friends(iq::Friends {
            xmlns: NAMESPACE_FRIENDS.into(),
            ..Default::default()
        })
        .into(),
    });
    session.send_stanza(friends_iq).await.unwrap();

    let server_response = session.recv_stanza().await.unwrap();
    let iq_response = match server_response {
        Stanza::Iq(iq) => iq,
        _ => panic!("invalid response from server {:?}", server_response),
    };
    let friends = match iq_response.payload {
        Some(iq::Payload::Friends(friends)) => friends,
        _ => panic!("invalid payload from server {:?}", iq_response.payload),
    };
    if let Some(list) = friends.friend_list {
        for friend in list {
            println!("\r< {} online", friend.to_string());
        }
    }
    println!("{}", "=".repeat(32));

    // Start sending and receiving messages
    session.start_messaging().await.unwrap();
}
