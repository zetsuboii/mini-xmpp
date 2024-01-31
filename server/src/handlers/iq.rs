use std::sync::Arc;

use parsers::{
    constants::NAMESPACE_FRIENDS,
    from_xml::WriteXmlString,
    stanza::iq::{Friends, Iq, IqPayload},
};
use tokio::sync::RwLock;

use crate::{session::Session, state::ServerState};
use color_eyre::eyre;

use super::HandleRequest;

impl HandleRequest for Iq {
    async fn handle_request(
        &self,
        current_session: &mut Session,
        state: Arc<RwLock<ServerState>>,
    ) -> eyre::Result<()> {
        if let Some(payload) = &self.payload {
            match payload {
                IqPayload::Friends(_) => handle_friends(&self.id, current_session, state).await?,
                _ => {
                    current_session
                        .connection
                        .send("unsupported IQ call".into())
                        .await?
                }
            }
        }
        Ok(())
    }
}

/// Handles "Friends" IQ call, which returns connected clients
async fn handle_friends(
    id: &str,
    current_session: &mut Session,
    state: Arc<RwLock<ServerState>>,
) -> eyre::Result<()> {
    let state = state.read().await;
    let current_resource = current_session.get_resource().unwrap();
    let current_jid = current_session.connection.get_jid().unwrap();

    // Filter out connections with different bare JIDs
    let mut friends = Vec::new();
    for (resource, session) in &state.sessions {
        if resource == &current_resource {
            continue;
        }

        let session = session.lock().await;
        if let Some(jid) = session.connection.get_jid() {
            if jid.bare() != current_jid.bare() {
                friends.push(jid.clone());
            }
        }
    }

    let mut iq = Iq::new(id.into());
    iq.type_ = Some("result".into());
    iq.payload = Some(IqPayload::Friends(Friends {
        xmlns: NAMESPACE_FRIENDS.into(),
        friend_list: Some(friends),
    }));

    current_session
        .connection
        .send(iq.write_xml_string()?)
        .await?;
    Ok(())
}
