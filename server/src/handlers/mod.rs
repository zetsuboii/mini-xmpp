mod iq;
mod message;
mod presence;

use std::sync::Arc;

use color_eyre::eyre;
use parsers::stanza::Stanza;
use tokio::sync::RwLock;

use crate::{session::Session, state::ServerState};

/// Represents a request made inside a session
/// Includes the session itself and the server state at the moment
pub struct Request<'se> {
    pub session: &'se mut Session,
    pub state: Arc<RwLock<ServerState>>,
}

impl<'se> Request<'se> {
    pub fn new(session: &'se mut Session, state: Arc<RwLock<ServerState>>) -> Self {
        Self { session, state }
    }
}

/// Trait implemented by structs that can be handled by a XMPP sesssion
pub trait HandleRequest<'se> {
    async fn handle_request(&self, request: &mut Request<'se>) -> eyre::Result<()>;
}

impl<'se> HandleRequest<'se> for Stanza {
    async fn handle_request(&self, request: &mut Request<'se>) -> eyre::Result<()> {
        match self {
            Stanza::Message(message) => message.handle_request(request).await,
            Stanza::Presence(presence) => presence.handle_request(request).await,
            Stanza::Iq(iq) => iq.handle_request(request).await,
        }
    }
}
