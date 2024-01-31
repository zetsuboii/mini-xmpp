use std::sync::Arc;

use color_eyre::eyre;
use parsers::{from_xml::WriteXmlString, jid::Jid, stanza::message::Message};
use tokio::sync::RwLock;

use crate::{session::Session, state::ServerState};

use super::HandleRequest;

impl HandleRequest for Message {
    async fn handle_request(
        &self,
        current_session: &mut Session,
        state: Arc<RwLock<ServerState>>,
    ) -> eyre::Result<()> {
        if let Some(jid) = &self.to {
            let jid = Jid::try_from(jid.clone())?;
            if let Some(resource) = jid.resource_part() {
                handle_message_with_res(&resource, self, current_session, state).await?;
            } else {
                handle_message(jid.bare().as_str(), self, current_session, state).await?;
            }
        }
        Ok(())
    }
}

/// Handles a message with resource bound
/// Only sends to the connection with given resource
async fn handle_message_with_res(
    resource: &str,
    message: &Message,
    current_session: &mut Session,
    state: Arc<RwLock<ServerState>>,
) -> eyre::Result<()> {
    let state = state.read().await;
    let current_resource = current_session.get_resource().unwrap();
    if resource == &current_resource {
        // Don't allow messagin oneself
        return Ok(());
    }

    match state.sessions.get(resource) {
        Some(session) => {
            let mut session = session.lock().await;
            session.connection.send(message.write_xml_string()?).await?;
        }
        None => {
            // Send error to the client
            current_session
                .connection
                .send("no such resource".into())
                .await?;
        }
    }
    Ok(())
}

/// Handles message with no resource
/// Sends to all connection with matching JIDs.
async fn handle_message(
    bare_jid: &str,
    message: &Message,
    current_session: &mut Session,
    state: Arc<RwLock<ServerState>>,
) -> eyre::Result<()> {
    let state = state.read().await;
    let current_resource = current_session.get_resource().unwrap();

    for (resource, session) in &state.sessions {
        if &current_resource == resource {
            // Skip current resource
            continue;
        }
        let mut session = session.lock().await;
        // Check if JID matches the expected jid
        let jid = session.connection.get_jid().map(|jid| jid.bare());
        if let Some(jid) = jid {
            if jid.as_str() == bare_jid {
                // If matches, send message
                session.connection.send(message.write_xml_string()?).await?;
            }
        }
    }
    Ok(())
}
