use self::message::Message;
use self::presence::Presence;
use self::iq::Iq;

mod message;
mod presence;
mod iq;

/// Basic unit of communication in XMPP.
/// They are the equivalent of HTTP requests and responses.
///
/// https://www.rfc-editor.org/rfc/rfc6120.html#section-8
#[derive(Debug, Clone)]
pub enum Stanza {
    Message(Message),
    Presence(Presence),
    Iq(Iq),
}
