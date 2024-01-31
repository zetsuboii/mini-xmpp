use color_eyre::eyre;
use parsers::{from_xml::WriteXmlString, jid::Jid, stanza::message::Message};

use super::{HandleRequest, Request};

impl<'se> HandleRequest<'se> for Message {
    async fn handle_request(&self, request: &mut Request<'se>) -> eyre::Result<()> {
        if let Some(jid) = &self.to {
            let jid = Jid::try_from(jid.clone())?;
            if let Some(resource) = jid.resource_part() {
                handle_message_with_res(&resource, self, request).await?;
            } else {
                handle_message(jid.bare().as_str(), self, request).await?;
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
    request: &mut Request<'_>,
) -> eyre::Result<()> {
    let state = request.state.read().await;
    let current_resource = request.session.get_resource().unwrap();
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
            request
                .session
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
    request: &mut Request<'_>,
) -> eyre::Result<()> {
    let state = request.state.read().await;
    let current_resource = request.session.get_resource().unwrap();

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
