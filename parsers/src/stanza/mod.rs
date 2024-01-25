use color_eyre::eyre;
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::from_xml::{ReadXml, WriteXml};

use self::iq::Iq;
use self::message::Message;
use self::presence::Presence;

pub  mod iq;
pub mod message;
pub mod presence;

/// Basic unit of communication in XMPP.
/// They are the equivalent of HTTP requests and responses.
///
/// https://www.rfc-editor.org/rfc/rfc6120.html#section-8
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stanza {
    Message(Message),
    Presence(Presence),
    Iq(Iq),
}

impl ReadXml<'_> for Stanza {
    fn read_xml<'a>(root: Event<'a>, reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        let start = match &root {
            Event::Start(tag) => tag,
            Event::Empty(tag) => tag,
            _ => eyre::bail!("invalid start event"),
        };

        match start.name().as_ref() {
            b"message" => Message::read_xml(root, reader).map(Stanza::Message),
            b"presence" => Presence::read_xml(root, reader).map(Stanza::Presence),
            b"iq" => Iq::read_xml(root, reader).map(Stanza::Iq),
            _ => eyre::bail!("invalid start tag"),
        }
    }
}

impl WriteXml for Stanza {
    fn write_xml(
        &self,
        writer: &mut quick_xml::Writer<std::io::Cursor<Vec<u8>>>,
    ) -> eyre::Result<()> {
        match self {
            Stanza::Message(message) => message.write_xml(writer),
            Stanza::Presence(presence) => presence.write_xml(writer),
            Stanza::Iq(iq) => iq.write_xml(writer),
        }
    }
}

#[cfg(test)]
mod tests {
    use tests::iq::{Friends, IqPayload};

    use crate::from_xml::ReadXmlString;

    use super::*;

    #[test]
    fn test_stanza_read() {
        let presence_xml = r#"<presence
            id='123'
            from='alice@mail.com'
            to='bob@mail.com'
        />"#;

        let stanza = Stanza::read_xml_string(presence_xml).unwrap();
        assert_eq!(
            stanza,
            Stanza::Presence(Presence {
                id: Some("123".to_string()),
                from: Some("alice@mail.com".to_string()),
                to: Some("bob@mail.com".to_string()),
            })
        );

        let message_xml = r#"
            <message 
                id='123'
                from='alice@mail.com'
                to='bob@mail.com'
                xml:lang='en'>
                    <body>hello</body>
            </message>
        "#;
        let stanza = Stanza::read_xml_string(message_xml).unwrap();
        assert_eq!(
            stanza,
            Stanza::Message(Message {
                id: Some("123".to_string()),
                from: Some("alice@mail.com".to_string()),
                to: Some("bob@mail.com".to_string()),
                xml_lang: Some("en".to_string()),
                body: Some("hello".to_string()),
            })
        );

        let iq_xml = r#"
            <iq 
                id='123'
                from='alice@mail.com'
                type='get'>
                    <friends xmlns='urn:example:friends'/>
            </iq>
        "#;

        let stanza = Stanza::read_xml_string(iq_xml).unwrap();
        assert_eq!(
            stanza,
            Stanza::Iq(Iq {
                id: "123".into(),
                from: Some("alice@mail.com".to_string()),
                type_: Some("get".to_string()),
                payload: Some(IqPayload::Friends(Friends {
                    xmlns: "urn:example:friends".to_string(),
                    ..Default::default()
                })),
            })
        );
    }
}
