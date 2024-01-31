use color_eyre::eyre;
use parsers::{from_xml::WriteXmlString, stanza::presence::Presence};

use super::{HandleRequest, Request};

impl<'se> HandleRequest<'se> for Presence {
    async fn handle_request(&self, request: &mut Request<'se>) -> eyre::Result<()> {
        // Send presence to all connected clients
        let state = request.state.read().await;
        let current_resource = request.session.get_resource().unwrap();
        for (resource, session) in &state.sessions {
            if &current_resource == resource {
                // Skip current session
                continue;
            } else {
                let mut session = session.lock().await;
                let jid = session.connection.get_jid();
                let current_jid = request.session.connection.get_jid();
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
