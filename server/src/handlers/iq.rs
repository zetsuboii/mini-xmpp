use parsers::{
    constants::NAMESPACE_FRIENDS,
    from_xml::WriteXmlString,
    stanza::iq::{Friends, Iq, IqPayload},
};

use color_eyre::eyre;

use super::{HandleRequest, Request};

impl<'se> HandleRequest<'se> for Iq {
    async fn handle_request(&self, request: &'se mut Request<'se>) -> eyre::Result<()> {
        if let Some(payload) = &self.payload {
            match payload {
                IqPayload::Friends(_) => handle_friends(&self.id, request).await?,
                _ => {
                    // Send error to the client
                    request
                        .session_mut()
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
    let state = request.state();
    let state = state.read().await;
    let current_resource = request.session_mut().get_resource().unwrap();
    let current_jid = request.session_mut().connection.get_jid().unwrap();

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

    request
        .session_mut()
        .connection
        .send(iq.write_xml_string()?)
        .await?;
    Ok(())
}
