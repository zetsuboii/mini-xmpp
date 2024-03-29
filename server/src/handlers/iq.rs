use parsers::{
    constants::NAMESPACE_FRIENDS,
    from_xml::WriteXmlString,
    stanza::iq::{Friends, Iq, Payload},
};

use color_eyre::eyre;

use super::{HandleRequest, Request};

impl<'se> HandleRequest<'se> for Iq {
    async fn handle_request(&self, request: &mut Request<'se>) -> eyre::Result<()> {
        if let Some(payload) = &self.payload {
            match payload {
                Payload::Friends(_) => handle_friends(&self.id, request).await?,
                _ => {
                    // Send error to the client
                    request
                        .session
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
async fn handle_friends(id: &str, request: &mut Request<'_>) -> eyre::Result<()> {
    let state = request.state.read().await;
    let current_resource = request.session.get_resource().unwrap();
    let current_jid = request.session.connection.get_jid().unwrap();

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
    iq.payload = Some(Payload::Friends(Friends {
        xmlns: NAMESPACE_FRIENDS.into(),
        friend_list: Some(friends),
    }));

    request
        .session
        .connection
        .send(iq.write_xml_string()?)
        .await?;
    Ok(())
}
