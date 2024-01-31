use std::io::Cursor;

use color_eyre::eyre;
use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Reader, Writer,
};

use crate::from_xml::{ReadXml, WriteXml};

/// XMPP address of the form <localpart@domainpart/resourcepart>
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Jid {
    pub local_part: String,
    pub domain_part: String,
    pub resource_part: Option<String>,
}

#[allow(unused)]
impl Jid {
    /// Creates a new JID
    ///
    /// ## Generic Types
    /// - `T`: Any type that can be turned into String
    /// - `U`: Any type that can be turned into String
    ///
    /// ## Params
    /// - `local_part`: Local part of the JID
    /// - `domain_part`: Domain part of the JID
    pub fn new<T, U>(local_part: T, domain_part: U) -> Self
    where
        T: Into<String>,
        U: Into<String>,
    {
        Self {
            local_part: local_part.into(),
            domain_part: domain_part.into(),
            ..Default::default()
        }
    }

    /// Adds resource
    ///
    /// ## Generic Types
    /// - `T`: Any type that can be turned into String
    ///
    /// ## Params
    /// - `resource_part`: Resource part of the JID
    pub fn with_resource<T>(mut self, resource_part: T) -> Self
    where
        T: Into<String>,
    {
        self.resource_part = Some(resource_part.into());
        self
    }

    pub fn local_part(&self) -> &str {
        self.local_part.as_ref()
    }

    pub fn domain_part(&self) -> &str {
        self.domain_part.as_ref()
    }

    pub fn resource_part(&self) -> Option<&String> {
        self.resource_part.as_ref()
    }

    /// Returns the bare JID without resource
    pub fn bare(&self) -> String {
        format!("{}@{}", self.local_part(), self.domain_part())
    }
}

impl TryFrom<String> for Jid {
    type Error = eyre::ErrReport;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let (local_part, mut rest) = if let Some(at) = value.find('@') {
            value.split_at(at)
        } else {
            eyre::bail!("@ not found");
        };

        rest = &rest[1..]; // Skip @

        if let Some(slash) = rest.find('/') {
            let (domain_part, rest) = rest.split_at(slash);
            let resource_part = &rest[1..]; // Skip /
            Ok(Jid::new(local_part, domain_part).with_resource(resource_part))
        } else {
            Ok(Jid::new(local_part, rest))
        }
    }
}

impl ToString for Jid {
    fn to_string(&self) -> String {
        match &self.resource_part {
            Some(resource_part) => {
                format!("{}@{}/{}", self.local_part, self.domain_part, resource_part)
            }
            None => format!("{}@{}", self.local_part, self.domain_part),
        }
    }
}

impl ReadXml<'_> for Jid {
    fn read_xml<'a>(start: Event<'a>, reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        let start = match start {
            Event::Start(tag) => tag,
            _ => eyre::bail!("invalid start tag"),
        };
        if start.name().as_ref() != b"jid" {
            eyre::bail!("invalid tag name")
        }

        // { jid }
        let text = match reader.read_event()? {
            Event::Text(text) => String::from_utf8(text.to_vec())?,
            _ => eyre::bail!("invalid text"),
        };

        // </jid>
        match reader.read_event()? {
            Event::End(tag) => {
                if tag.name().as_ref() != b"jid" {
                    eyre::bail!("invalid end tag")
                }
            }
            _ => eyre::bail!("invalid end tag"),
        }

        Self::try_from(text)
    }
}

impl WriteXml for Jid {
    fn write_xml(&self, writer: &mut Writer<Cursor<Vec<u8>>>) -> eyre::Result<()> {
        // <jid>
        writer.write_event(Event::Start(BytesStart::new("jid")))?;
        // { jid }
        writer.write_event(Event::Text(BytesText::new(&self.to_string())))?;
        // </jid>
        writer.write_event(Event::End(BytesEnd::new("jid")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::from_xml::{ReadXmlString, WriteXmlString};

    use super::*;

    #[test]
    fn serialize_without_resource() {
        let jid = Jid::new("user", "mail.com");
        let serialized = jid.write_xml_string().unwrap();
        assert_eq!(serialized, "<jid>user@mail.com</jid>");
    }

    #[test]
    fn serialize_with_resource() {
        let jid = Jid::new("user", "mail.com").with_resource("my-resource");
        let serialized = jid.write_xml_string().unwrap();
        assert_eq!(serialized, "<jid>user@mail.com/my-resource</jid>");
    }

    #[test]
    fn deserialize_without_resource() {
        let raw = "<jid>user@mail.com</jid>";
        let jid = Jid::read_xml_string(raw).unwrap();
        assert_eq!(jid.local_part(), "user");
        assert_eq!(jid.domain_part(), "mail.com");
        assert_eq!(jid.resource_part(), None);
    }

    #[test]
    fn deserialize_with_resource() {
        let raw = "<jid>user@mail.com/my-resource</jid>";
        let jid = Jid::read_xml_string(raw).unwrap();
        assert_eq!(jid.local_part(), "user");
        assert_eq!(jid.domain_part(), "mail.com");
        assert_eq!(jid.resource_part(), Some(&"my-resource".to_string()));
    }
}
