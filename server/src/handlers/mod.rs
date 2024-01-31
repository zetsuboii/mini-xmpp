mod iq;
mod message;
mod presence;

use std::sync::Arc;

use color_eyre::eyre;
use parsers::stanza::Stanza;
use tokio::sync::RwLock;

use crate::{session::Session, state::ServerState};

/// Trait implemented by structs that can be handled by a XMPP sesssion
pub trait HandleRequest {
    async fn handle_request(
        &self,
        session: &mut Session,
        state: Arc<RwLock<ServerState>>,
    ) -> eyre::Result<()>;
}

impl HandleRequest for Stanza {
    async fn handle_request(
        &self,
        session: &mut Session,
        state: Arc<RwLock<ServerState>>,
    ) -> eyre::Result<()> {
        match self {
            Stanza::Message(message) => message.handle_request(session, state).await,
            Stanza::Presence(presence) => presence.handle_request(session, state).await,
            Stanza::Iq(iq) => iq.handle_request(session, state).await,
        }
    }
}
