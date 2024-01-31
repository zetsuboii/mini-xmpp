use std::sync::Arc;

use color_eyre::eyre;
use parsers::{from_xml::WriteXmlString, stanza::presence::Presence};
use tokio::sync::RwLock;

use crate::{session::Session, state::ServerState};

use super::HandleRequest;

impl HandleRequest for Presence {
    async fn handle_request(
        &self,
        current_session: &mut Session,
        state: Arc<RwLock<ServerState>>,
    ) -> eyre::Result<()> {
        // Send presence to all connected clients
        let state = state.read().await;
        let current_resource = current_session.get_resource().unwrap();
        for (resource, session) in &state.sessions {
            if &current_resource == resource {
                // Skip current session
                continue;
            } else {
                let mut session = session.lock().await;
                let jid = session.connection.get_jid();
                let current_jid = current_session.connection.get_jid();
                if let (Some(jid), Some(current_jid)) = (jid, current_jid) {
                    if jid.bare() == current_jid.bare() {
                        continue;
                    }
                }
                session.connection.send(self.write_xml_string()?).await?;
            }
        }
        Ok(())
    }
}
