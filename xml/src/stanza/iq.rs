use std::io::Cursor;

use color_eyre::eyre;
use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    name::QName,
    Reader, Writer,
};

use crate::{
    empty::IsEmpty,
    from_xml::{ReadXml, WriteXml},
    jid::Jid,
    utils::try_get_attribute,
};

/// Represents an IQ stanza in XMPP, which is used for sending queries or
/// commands and receiving responses.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Iq {
    pub id: Option<String>,
    pub from: Option<String>,
    pub type_: Option<String>,
    pub payload: Option<IqPayload>,
}

impl Iq {
    pub fn new() -> Self {
        Default::default()
    }
}

impl ReadXml<'_> for Iq {
    fn read_xml(reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        let iq_start = match reader.read_event()? {
            Event::Start(tag) => tag,
            Event::Empty(tag) => tag,
            _ => eyre::bail!("invalid start event"),
        };

        Self::read_xml_from_start(iq_start, reader)
    }

    fn read_xml_from_start<'a>(
        start: BytesStart<'a>,
        reader: &mut Reader<&[u8]>,
    ) -> eyre::Result<Self> {
        if start.name().as_ref() != b"iq" {
            eyre::bail!("invalid start tag")
        }

        let mut result = Self::new();
        result.id = try_get_attribute(&start, "id").ok();
        result.from = try_get_attribute(&start, "from").ok();
        result.type_ = try_get_attribute(&start, "type").ok();

        while let Ok(event) = reader.read_event() {
            match event {
                Event::Start(tag) => match tag.name().as_ref() {
                    // <bind>
                    b"bind" => {
                        result.payload =
                            Some(IqPayload::Bind(Bind::read_xml_from_start(tag, reader)?))
                    }
                    // <friends>
                    b"friends" => {
                        result.payload = Some(IqPayload::Friends(Friends::read_xml_from_start(
                            tag, reader,
                        )?))
                    }
                    _ => eyre::bail!("invalid tag name"),
                },
                Event::End(tag) => {
                    if tag.name().as_ref() != b"iq" {
                        eyre::bail!("invalid end tag")
                    }
                    break;
                }
                Event::Eof => eyre::bail!("unexpected EOF"),
                _ => {}
            }
        }

        Ok(result)
    }
}

impl WriteXml for Iq {
    fn write_xml(&self, writer: &mut Writer<Cursor<Vec<u8>>>) -> eyre::Result<()> {
        let mut iq_start = BytesStart::new("iq");
        if let Some(id) = &self.id {
            iq_start.push_attribute(("id", id.as_str()));
        }
        if let Some(from) = &self.from {
            iq_start.push_attribute(("from", from.as_str()));
        }
        if let Some(type_) = &self.type_ {
            iq_start.push_attribute(("type", type_.as_str()));
        }

        if let Some(payload) = &self.payload {
            // <iq>
            writer.write_event(Event::Start(iq_start))?;

            // <bind>
            payload.write_xml(writer)?;

            // </iq>
            writer.write_event(Event::End(BytesEnd::new("iq")))?;
        } else {
            // <iq />
            writer.write_event(Event::Empty(iq_start))?;
        }

        Ok(())
    }
}

/// Possible payloads for an IQ stanza.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IqPayload {
    Bind(Bind),
    Friends(Friends),
}

impl ReadXml<'_> for IqPayload {
    fn read_xml(reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        let payload_start = match reader.read_event()? {
            Event::Start(tag) => tag,
            Event::Empty(tag) => tag,
            _ => eyre::bail!("invalid start event"),
        };

        Self::read_xml_from_start(payload_start, reader)
    }

    fn read_xml_from_start<'a>(
        start: BytesStart<'a>,
        reader: &mut quick_xml::Reader<&[u8]>,
    ) -> color_eyre::eyre::Result<Self> {
        match start.name().as_ref() {
            b"bind" => Ok(Self::Bind(Bind::read_xml_from_start(start, reader)?)),
            b"friends" => Ok(Self::Friends(Friends::read_xml_from_start(start, reader)?)),
            _ => eyre::bail!("invalid tag name"),
        }
    }
}

impl WriteXml for IqPayload {
    fn write_xml(&self, writer: &mut Writer<Cursor<Vec<u8>>>) -> eyre::Result<()> {
        match self {
            Self::Bind(bind) => bind.write_xml(writer),
            Self::Friends(friends) => friends.write_xml(writer),
        }
    }
}

//
// bind
//

/// Represents the 'bind' element in XMPP, which is used for resource binding
/// during session establishment.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Bind {
    pub xmlns: String,
    pub jid: Option<Jid>,
    pub resource: Option<String>,
}

impl Bind {
    pub fn new(xmlns: String) -> Self {
        Self {
            xmlns,
            ..Default::default()
        }
    }
}

impl IsEmpty for Bind {
    fn is_empty(&self) -> bool {
        self.jid.is_none() && self.resource.is_none()
    }
}

impl ReadXml<'_> for Bind {
    fn read_xml(reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        let bind_start = match reader.read_event()? {
            Event::Start(tag) => tag,
            Event::Empty(tag) => tag,
            _ => eyre::bail!("invalid start event"),
        };

        Self::read_xml_from_start(bind_start, reader)
    }

    fn read_xml_from_start<'a>(
        start: BytesStart<'a>,
        reader: &mut quick_xml::Reader<&[u8]>,
    ) -> color_eyre::eyre::Result<Self> {
        if start.name().as_ref() != b"bind" {
            eyre::bail!("invalid start tag")
        }

        let xmlns = try_get_attribute(&start, "xmlns")?;
        let mut result = Self::new(xmlns);

        while let Ok(event) = reader.read_event() {
            match event {
                Event::Start(tag) => match tag.name().as_ref() {
                    // <jid>
                    b"jid" => result.jid = Some(Jid::read_xml_from_start(tag, reader)?),
                    // <resource>
                    b"resource" => {
                        let resource = reader
                            .read_text(QName(b"resource"))
                            .map(|res| res.trim().to_string())?;
                        result.resource = Some(resource);
                    }
                    _ => eyre::bail!("invalid tag name"),
                },
                // </bind>
                Event::End(tag) => {
                    if tag.name().as_ref() != b"bind" {
                        eyre::bail!("invalid end tag")
                    }
                    break;
                }
                Event::Eof => eyre::bail!("unexpected EOF"),
                _ => {}
            }
        }

        Ok(result)
    }
}

impl WriteXml for Bind {
    fn write_xml(&self, writer: &mut Writer<Cursor<Vec<u8>>>) -> eyre::Result<()> {
        let mut bind_start = BytesStart::new("bind");
        bind_start.push_attribute(("xmlns", self.xmlns.as_ref()));

        if self.is_empty() {
            // <bind />
            writer.write_event(Event::Empty(bind_start))?;
        } else {
            // <bind>
            writer.write_event(Event::Start(bind_start))?;

            // <jid>
            if let Some(jid) = &self.jid {
                jid.write_xml(writer)?;
            }

            // <resource>
            if let Some(resource) = &self.resource {
                writer.write_event(Event::Start(BytesStart::new("resource")))?;
                writer.write_event(Event::Text(BytesText::new(resource.as_str())))?;
                writer.write_event(Event::End(BytesEnd::new("resource")))?;
            }

            // </bind>
            writer.write_event(Event::End(BytesEnd::new("bind")))?;
        }

        Ok(())
    }
}

//
// friends
//

/// Represents a custom 'friends' element, used to get friends list of a user.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Friends {
    pub xmlns: String,
    pub friend_list: Option<Vec<Jid>>,
}

impl Friends {
    pub fn new(xmlns: String) -> Self {
        Self {
            xmlns,
            ..Default::default()
        }
    }
}

impl ReadXml<'_> for Friends {
    fn read_xml(reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        let friends_start = match reader.read_event()? {
            Event::Start(tag) => tag,
            Event::Empty(tag) => tag,
            _ => eyre::bail!("invalid start event"),
        };

        Self::read_xml_from_start(friends_start, reader)
    }

    fn read_xml_from_start<'a>(
        start: BytesStart<'a>,
        reader: &mut quick_xml::Reader<&[u8]>,
    ) -> color_eyre::eyre::Result<Self> {
        if start.name().as_ref() != b"friends" {
            eyre::bail!("invalid start tag")
        }

        let xmlns = try_get_attribute(&start, "xmlns")?;
        let mut result = Self::new(xmlns);

        while let Ok(event) = reader.read_event() {
            // <jid>
            match event {
                Event::Start(tag) => {
                    let jid = Jid::read_xml_from_start(tag, reader)?;
                    match result.friend_list.as_mut() {
                        Some(list) => list.push(jid),
                        None => result.friend_list = Some(vec![jid]),
                    };
                }
                Event::End(tag) => {
                    if tag.name().as_ref() != b"friends" {
                        eyre::bail!("invalid end tag")
                    }
                    break;
                }
                Event::Eof => eyre::bail!("unexpected EOF"),
                _ => {}
            }
        }

        Ok(result)
    }
}

impl WriteXml for Friends {
    fn write_xml(&self, writer: &mut Writer<Cursor<Vec<u8>>>) -> eyre::Result<()> {
        let mut friends_start = BytesStart::new("friends");
        friends_start.push_attribute(("xmlns", self.xmlns.as_ref()));

        if let Some(friend_list) = &self.friend_list {
            // <friends>
            writer.write_event(Event::Start(friends_start))?;

            for friend in friend_list {
                friend.write_xml(writer)?;
            }

            // </friends>
            writer.write_event(Event::End(BytesEnd::new("friends")))?;
        } else {
            // <friends />
            writer.write_event(Event::Empty(friends_start))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::from_xml::{ReadXmlString, WriteXmlString};

    use super::*;

    #[test]
    fn test_iq() {
        let xml = r#"<iq id="123" from="alice@mail" type="set">
            <bind xmlns="urn:ietf:params:xml:ns:xmpp-bind">
                <jid> alice@mail.com </jid>
                <resource> phone </resource>
            </bind>
        </iq>"#;

        let iq = Iq::read_xml_string(xml).unwrap();
        assert_eq!(
            iq,
            Iq {
                id: Some("123".to_string()),
                from: Some("alice@mail".to_string()),
                type_: Some("set".to_string()),
                payload: Some(IqPayload::Bind(Bind {
                    xmlns: "urn:ietf:params:xml:ns:xmpp-bind".to_string(),
                    jid: Some(Jid::new("alice", "mail.com")),
                    resource: Some("phone".to_string()),
                })),
            }
        );
    }

    #[test]
    fn test_iq_payload() {
        let xml = r#"<bind xmlns="urn:ietf:params:xml:ns:xmpp-bind">
            <jid> alice@mail.com </jid>
            <resource> phone </resource>
        </bind>"#;

        let payload = IqPayload::read_xml_string(xml).unwrap();
        assert_eq!(
            payload,
            IqPayload::Bind(Bind {
                xmlns: "urn:ietf:params:xml:ns:xmpp-bind".to_string(),
                jid: Some(Jid::new("alice", "mail.com")),
                resource: Some("phone".to_string()),
            })
        );
    }

    #[test]
    fn test_bind() {
        let xml = r#"<bind xmlns="urn:ietf:params:xml:ns:xmpp-bind">
            <jid>alice@mail.com</jid>
            <resource>phone</resource>
        </bind>"#;

        let bind = Bind::read_xml_string(xml).unwrap();
        assert_eq!(
            bind,
            Bind {
                xmlns: "urn:ietf:params:xml:ns:xmpp-bind".to_string(),
                jid: Some(Jid::new("alice", "mail.com")),
                resource: Some("phone".to_string()),
            }
        );

        let mut bind = Bind::new("urn:ietf:params:xml:ns:xmpp-bind".to_string());
        bind.jid = Some(Jid::new("zet", "mail"));
        bind.resource = Some("phone".to_string());
        let xml = bind.write_xml_string().unwrap();
        assert_eq!(
            xml,
            [
                "<bind xmlns=\"urn:ietf:params:xml:ns:xmpp-bind\">",
                "<jid>zet@mail</jid>",
                "<resource>phone</resource>",
                "</bind>"
            ]
            .concat()
        );
    }

    #[test]
    fn test_friends() {
        let xml = r#"<friends xmlns="mini.jabber.com/friends">
            <jid> alice@mail.com/phone </jid>
            <jid> bob@mail.com/phone </jid>
        </friends>"#;

        let friends = Friends::read_xml_string(xml).unwrap();
        assert_eq!(
            friends,
            Friends {
                xmlns: "mini.jabber.com/friends".to_string(),
                friend_list: Some(vec![
                    Jid::new("alice", "mail.com").with_resource("phone"),
                    Jid::new("bob", "mail.com").with_resource("phone"),
                ]),
            }
        );
    }

    #[test]
    fn test_fail_friends() {
        // Fail when there's no end tag
        let xml = r#"<friends xmlns="mini.jabber.com/friends">
            <jid> alice@mail.com/phone </jid>
            <jid> bob@mail.com/phone </jid>
        "#;

        let friends = Friends::read_xml_string(xml);
        assert!(friends.is_err());
    }
}
