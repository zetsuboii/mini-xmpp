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
    pub id: String,
    pub from: Option<String>,
    pub type_: Option<String>,
    pub payload: Option<Payload>,
}

impl Iq {
    pub fn new(id: String) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }
}

impl ReadXml<'_> for Iq {
    fn read_xml<'a>(root: Event<'a>, reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        let (start, empty) = match root {
            Event::Empty(tag) => (tag, true),
            Event::Start(tag) => (tag, false),
            _ => eyre::bail!("invalid start event"),
        };
        if start.name().as_ref() != b"iq" {
            eyre::bail!("invalid start tag")
        }

        let id = try_get_attribute(&start, "id")?;
        let mut result = Self::new(id);

        result.from = try_get_attribute(&start, "from").ok();
        result.type_ = try_get_attribute(&start, "type").ok();

        if empty {
            return Ok(result);
        }

        while let Ok(event) = reader.read_event() {
            match event {
                Event::Empty(ref tag) | Event::Start(ref tag) => match tag.name().as_ref() {
                    // <bind> or <bind/>
                    b"bind" => {
                        result.payload =
                            Bind::read_xml(event, reader).map(Payload::Bind).map(Some)?
                    }
                    // <friends> or <friends/>
                    b"friends" => {
                        result.payload = Friends::read_xml(event, reader)
                            .map(Payload::Friends)
                            .map(Some)?
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
        iq_start.push_attribute(("id", self.id.as_str()));

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
pub enum Payload {
    Bind(Bind),
    Friends(Friends),
}

impl ReadXml<'_> for Payload {
    fn read_xml<'a>(
        root: Event<'a>,
        reader: &mut quick_xml::Reader<&[u8]>,
    ) -> color_eyre::eyre::Result<Self> {
        let start = match &root {
            Event::Start(tag) => tag,
            Event::Empty(tag) => tag,
            _ => eyre::bail!("invalid start event"),
        };

        match start.name().as_ref() {
            b"bind" => Ok(Self::Bind(Bind::read_xml(root, reader)?)),
            b"friends" => Ok(Self::Friends(Friends::read_xml(root, reader)?)),
            _ => eyre::bail!("invalid tag name"),
        }
    }
}

impl WriteXml for Payload {
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
    fn read_xml<'a>(
        root: Event<'a>,
        reader: &mut quick_xml::Reader<&[u8]>,
    ) -> color_eyre::eyre::Result<Self> {
        let (start, empty) = match root {
            Event::Empty(tag) => (tag, true),
            Event::Start(tag) => (tag, false),
            _ => eyre::bail!("invalid start event"),
        };
        if start.name().as_ref() != b"bind" {
            eyre::bail!("invalid start tag")
        }

        let xmlns = try_get_attribute(&start, "xmlns")?;
        let mut result = Self::new(xmlns);

        if empty {
            return Ok(result);
        }

        while let Ok(event) = reader.read_event() {
            match event {
                Event::Start(ref tag) => match tag.name().as_ref() {
                    // <jid>
                    b"jid" => result.jid = Some(Jid::read_xml(event, reader)?),
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
    fn read_xml<'a>(
        root: Event<'a>,
        reader: &mut quick_xml::Reader<&[u8]>,
    ) -> color_eyre::eyre::Result<Self> {
        if let Event::Empty(tag) = root {
            if tag.name().as_ref() != b"friends" {
                eyre::bail!("invalid start tag")
            }

            let xmlns = try_get_attribute(&tag, "xmlns")?;
            return Ok(Self::new(xmlns));
        }

        let start = match root {
            Event::Start(tag) => {
                if tag.name().as_ref() == b"friends" {
                    tag
                } else {
                    eyre::bail!("invalid start tag")
                }
            }
            _ => eyre::bail!("invalid start event"),
        };

        let xmlns = try_get_attribute(&start, "xmlns")?;
        let mut result = Self::new(xmlns);

        while let Ok(event) = reader.read_event() {
            // <jid>
            match event {
                Event::Start(_) => {
                    let jid = Jid::read_xml(event, reader)?;
                    match result.friend_list.as_mut() {
                        Some(list) => list.push(jid),
                        None => result.friend_list = Some(vec![jid]),
                    };
                }
                Event::End(tag) => {
                    if tag.name().as_ref() != b"friends" {
                        eyre::bail!("invalid end tag {:?}", tag.name())
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
                id: "123".to_string(),
                from: Some("alice@mail".to_string()),
                type_: Some("set".to_string()),
                payload: Some(Payload::Bind(Bind {
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

        let payload = Payload::read_xml_string(xml).unwrap();
        assert_eq!(
            payload,
            Payload::Bind(Bind {
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
