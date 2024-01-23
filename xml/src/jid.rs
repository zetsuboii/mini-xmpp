use color_eyre::eyre;
use quick_xml::{de::from_str, se::to_string};
use serde::{Deserialize, Serialize};

use crate::from_xml::{FromXml, ToXml};

/// XMPP address of the form <localpart@domainpart/resourcepart>
#[derive(Debug, Clone, Default)]
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

/// JID representation in XML
#[derive(Serialize, Deserialize)]
#[serde(rename = "jid")]
struct JidXml {
    #[serde(rename = "$text")]
    pub inner: String,
}

impl Serialize for Jid {
    /// Custom serializer for Jid struct
    ///
    /// This is required as we'll combine local, domain and resource parts
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let jid_concat = match &self.resource_part {
            Some(resource_part) => {
                format!("{}@{}/{}", self.local_part, self.domain_part, resource_part)
            }
            None => format!("{}@{}", self.local_part, self.domain_part),
        };
        JidXml { inner: jid_concat }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Jid {
    /// Custom deserializer for Jid struct
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO: There's no trivial way to map eyre::ErrReport to D::Error
        JidXml::deserialize(deserializer).map(|jid| Jid::try_from(jid.inner).unwrap())
    }
}

impl ToXml for Jid {
    fn to_xml(&self) -> String {
        to_string(self).expect("failed to convert to string")
    }
}

impl<T: AsRef<str>> FromXml<T> for Jid {
    type Out = Self;

    fn from_xml(value: T) -> eyre::Result<Self> {
        from_str(value.as_ref())
            .map(|v| v)
            .map_err(|_| eyre::eyre!("failed to decode"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::{de::from_str, se::to_string};

    #[test]
    fn serialize_without_resource() {
        let jid = Jid::new("user", "mail.com");
        let serialized = to_string(&jid).unwrap();
        assert_eq!(serialized, "<jid>user@mail.com</jid>");
    }

    #[test]
    fn serialize_with_resource() {
        let jid = Jid::new("user", "mail.com").with_resource("my-resource");
        let serialized = to_string(&jid).unwrap();
        assert_eq!(serialized, "<jid>user@mail.com/my-resource</jid>");
    }

    #[test]
    fn deserialize_without_resource() {
        let raw = "<jid>user@mail.com</jid>";
        let jid: Jid = from_str(raw).unwrap();
        assert_eq!(jid.local_part(), "user");
        assert_eq!(jid.domain_part(), "mail.com");
        assert_eq!(jid.resource_part(), None);
    }

    #[test]
    fn deserialize_with_resource() {
        let raw = "<jid>user@mail.com/my-resource</jid>";
        let jid: Jid = from_str(raw).unwrap();
        assert_eq!(jid.local_part(), "user");
        assert_eq!(jid.domain_part(), "mail.com");
        assert_eq!(jid.resource_part(), Some(&"my-resource".to_string()));
    }
}
