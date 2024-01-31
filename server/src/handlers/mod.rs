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
    session: &'se mut Session,
    state: Arc<RwLock<ServerState>>,
}

impl<'se> Request<'se> {
    pub fn new(session: &'se mut Session, state: Arc<RwLock<ServerState>>) -> Self {
        Self { session, state }
    }

    pub fn session_mut<'a: 'se>(&'a mut self) -> &'se mut Session {
        self.session
    }

    pub fn state<'st>(&self) -> &'st RwLock<ServerState> {
        &self.state
    }
}

/// Trait implemented by structs that can be handled by a XMPP sesssion
pub trait HandleRequest<'s> {
    async fn handle_request(&self, request: &'s mut Request<'s>) -> eyre::Result<()>;
}

impl<'s> HandleRequest<'s> for Stanza {
    async fn handle_request(&self, request: &'s mut Request<'s>) -> eyre::Result<()> {
        match self {
            Stanza::Message(message) => message.handle_request(request).await,
            Stanza::Presence(presence) => presence.handle_request(request).await,
            Stanza::Iq(iq) => iq.handle_request(request).await,
        }
    }
}
