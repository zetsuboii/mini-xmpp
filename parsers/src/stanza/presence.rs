use std::io::Cursor;

use color_eyre::eyre;
use quick_xml::{
    events::{BytesStart, Event},
    name::QName,
    Reader, Writer,
};

use crate::{
    from_xml::{ReadXml, WriteXml},
    utils::try_get_attribute,
};

/// Presence information for a XMPP user
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Presence {
    pub id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

impl Presence {
    pub fn new() -> Presence {
        Default::default()
    }
}

impl ReadXml<'_> for Presence {
    fn read_xml<'a>(event: Event<'a>, reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        let (start, empty) = match event {
            Event::Empty(tag) => (tag, true),
            Event::Start(tag) => (tag, false),
            _ => eyre::bail!("invalid start tag"),
        };
        if start.name().as_ref() != b"presence" {
            eyre::bail!("invalid start tag");
        }

        let mut presence = Self::new();
        presence.id = try_get_attribute(&start, "id").ok();
        presence.from = try_get_attribute(&start, "from").ok();
        presence.to = try_get_attribute(&start, "to").ok();

        // If not empty tag, read until end tag
        if !empty {
            reader.read_to_end(QName(b"presence"))?;
        }

        Ok(presence)
    }
}

impl WriteXml for Presence {
    fn write_xml(&self, writer: &mut Writer<Cursor<Vec<u8>>>) -> eyre::Result<()> {
        // <presence/>
        let mut presence_start = BytesStart::new("presence");

        if let Some(id) = &self.id {
            presence_start.push_attribute(("id", id.as_str()));
        }

        if let Some(from) = &self.from {
            presence_start.push_attribute(("from", from.as_str()));
        }

        if let Some(to) = &self.to {
            presence_start.push_attribute(("to", to.as_str()));
        }

        writer.write_event(Event::Empty(presence_start))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::from_xml::{ReadXmlString, WriteXmlString};

    use super::*;

    #[test]
    fn test_presence_empty() {
        let presence: Presence = Presence::new();

        let serialized = presence.write_xml_string().unwrap();
        assert_eq!(serialized, "<presence/>");

        let presence: Presence = Presence::read_xml_string(serialized.as_str()).unwrap();
        assert_eq!(presence, Presence::new());
    }

    #[test]
    fn test_presence() {
        let mut presence: Presence = Presence::new();
        presence.id = Some("123".to_string());
        presence.from = Some("alice@mail.com/phone".to_string());
        presence.to = Some("bob@mail.com/phone".to_string());

        let serialized = presence.write_xml_string().unwrap();
        assert_eq!(
            serialized,
            [
                "<presence ",
                "id=\"123\" ",
                "from=\"alice@mail.com/phone\" ",
                "to=\"bob@mail.com/phone\"/>",
            ]
            .concat()
        );

        let presence: Presence = Presence::read_xml_string(serialized.as_str()).unwrap();
        assert_eq!(presence, presence);
    }
}
