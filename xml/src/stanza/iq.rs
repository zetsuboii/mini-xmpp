use std::{fmt::Write, io::Cursor};

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

#[derive(Debug, Clone)]
pub struct Iq {
    pub id: Option<String>,
    pub from: Option<String>,
    pub type_: Option<String>,
    pub payload: IqPayload,
}

#[derive(Debug, Clone)]
pub enum IqPayload {
    Bind(Bind),
    Friends(Friends),
}

//
// bind
//

#[derive(Default, Debug, Clone)]
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

#[derive(Default, Debug, Clone)]
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
    fn test_bind() {
        let xml = r#"<bind xmlns="urn:ietf:params:xml:ns:xmpp-bind">
            <jid>alice@mail.com</jid>
            <resource>phone</resource>
        </bind>"#;

        let bind = Bind::read_xml_string(xml).unwrap();
        assert_eq!(bind.xmlns, "urn:ietf:params:xml:ns:xmpp-bind");
        assert_eq!(bind.jid.unwrap().to_string(), "alice@mail.com".to_string());
        assert_eq!(bind.resource, Some("phone".to_string()));

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
            ].concat()
        );
    }

    #[test]
    fn test_friends() {
        let xml = r#"<friends xmlns="mini.jabber.com/friends">
            <jid> alice@mail.com/phone </jid>
            <jid> bob@mail.com/phone </jid>
        </friends>"#;

        let friends = Friends::read_xml_string(xml).unwrap();
        assert_eq!(friends.xmlns, "mini.jabber.com/friends");

        let friend_list = friends.friend_list.unwrap();
        let alice = &friend_list[0];
        assert_eq!(alice.local_part(), "alice");
        assert_eq!(alice.domain_part(), "mail.com");
        assert_eq!(alice.resource_part(), Some(&"phone".to_string()));
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
