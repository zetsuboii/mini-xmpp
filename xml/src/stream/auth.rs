//! Authentication structures and methods for XML streams.
#![allow(unused)]

use std::io::Cursor;

use crate::{
    from_xml::{ReadXml, WriteXml},
    utils::try_get_attribute,
};
use base64::{prelude::BASE64_STANDARD as BASE64, Engine};
use color_eyre::eyre;
use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Reader,
};

use super::features::Mechanism;

//
// authentication request
//

#[derive(Debug, Clone)]
pub struct AuthRequest {
    pub xmlns: String,
    pub mechanism: Mechanism,
    pub value: String,
}

impl AuthRequest {
    pub fn new(xmlns: String, mechanism: Mechanism, value: String) -> Self {
        Self {
            xmlns,
            mechanism,
            value,
        }
    }
}

impl ReadXml<'_> for AuthRequest {
    fn read_xml(reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        // <auth xmlns="urn:ietf:params:xml:ns:xmpp-sasl" mechanism="PLAIN">
        let auth_start = match reader.read_event()? {
            Event::Start(tag) => tag,
            _ => eyre::bail!("invalid xml"),
        };
        Self::read_xml_from_start(auth_start, reader)
    }

    fn read_xml_from_start<'a>(
        start: BytesStart<'a>,
        reader: &mut Reader<&[u8]>,
    ) -> eyre::Result<Self> {
        if start.name().as_ref() != b"auth" {
            eyre::bail!("invalid tag name")
        }

        let xmlns = try_get_attribute(&start, "xmlns")?;
        let mechanism = try_get_attribute(&start, "mechanism")
            .and_then(|mechanism| Mechanism::try_from(mechanism.as_str()))?;

        let mut value = None;

        while let Ok(event) = reader.read_event() {
            match event {
                Event::Text(text) => {
                    value = Some(String::from_utf8(text.as_ref().into())?);
                }
                Event::End(tag) => {
                    if tag.name().as_ref() != b"auth" {
                        eyre::bail!("invalid tag name")
                    }
                    break;
                }
                Event::Eof => eyre::bail!("unexpected EOF"),
                _ => {}
            }
        }

        Ok(AuthRequest {
            xmlns,
            mechanism,
            value: value.ok_or(eyre::eyre!("missing value"))?,
        })
    }
}

impl WriteXml for AuthRequest {
    fn write_xml(&self, writer: &mut quick_xml::Writer<Cursor<Vec<u8>>>) -> eyre::Result<()> {
        // <auth xmlns="urn:ietf:params:xml:ns:xmpp-sasl" mechanism="PLAIN">
        let mut auth_start = BytesStart::new("auth");
        auth_start.push_attribute(("xmlns", self.xmlns.as_ref()));
        auth_start.push_attribute(("mechanism", self.mechanism.to_string().as_str()));
        writer.write_event(Event::Start(auth_start))?;

        // {...}
        writer.write_event(Event::Text(BytesText::new(self.value.as_ref())))?;

        // </auth>
        writer.write_event(Event::End(BytesEnd::new("auth")))?;
        Ok(())
    }
}

//
// authentication success
//

#[derive(Debug, Clone)]
pub struct AuthSuccess {
    pub xmlns: String,
}

impl AuthSuccess {
    pub fn new(xmlns: String) -> Self {
        Self { xmlns }
    }
}

impl ReadXml<'_> for AuthSuccess {
    fn read_xml(reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        // <success xmlns/>
        let success_start = match reader.read_event()? {
            Event::Empty(tag) => tag,
            _ => eyre::bail!("invalid xml"),
        };
        Self::read_xml_from_start(success_start, reader)
    }

    fn read_xml_from_start<'a>(
        start: BytesStart<'a>,
        _reader: &mut Reader<&[u8]>,
    ) -> eyre::Result<Self> {
        if start.name().as_ref() != b"success" {
            eyre::bail!("invalid tag name")
        }

        let xmlns = try_get_attribute(&start, "xmlns")?;

        Ok(AuthSuccess { xmlns })
    }
}

impl WriteXml for AuthSuccess {
    fn write_xml(&self, writer: &mut quick_xml::Writer<Cursor<Vec<u8>>>) -> eyre::Result<()> {
        // <success xmlns />
        let mut success_start = BytesStart::new("success");
        success_start.push_attribute(("xmlns", self.xmlns.as_ref()));
        writer.write_event(Event::Empty(success_start))?;
        Ok(())
    }
}

//
// plaintext credentials
//

#[derive(Debug)]
pub struct PlaintextCredentials {
    pub username: String,
    pub password: String,
}

impl PlaintextCredentials {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub fn from_base64(value: String) -> eyre::Result<Self> {
        let value = BASE64.decode(value.as_bytes())?;
        let value = std::str::from_utf8(&value)?;
        let mut values: Vec<String> = value.split("\0").map(|s| s.to_string()).collect();
        println!("{:?}", values);
        let password = values.pop().ok_or(eyre::eyre!("missing password"))?;
        let username = values.pop().ok_or(eyre::eyre!("missing username"))?;
        Ok(Self::new(username, password))
    }

    pub fn to_base64(&self) -> String {
        let mut serialized = String::new();
        serialized.push_str(&self.username.as_str());
        serialized.push('\0');
        serialized.push_str(&self.password.as_str());
        BASE64.encode(serialized)
    }
}

#[cfg(test)]
mod tests {
    use crate::from_xml::ReadXmlString;

    use super::*;

    #[test]
    fn test_auth_request() -> eyre::Result<()> {
        let xml = r#"
            <auth xmlns='urn:ietf:params:xml:ns:xmpp-sasl' mechanism='PLAIN'>
                AGp1bGlldAByMG0zMG15cjBtMzA=
            </auth>
        "#;
        let auth = AuthRequest::read_xml_string(xml)?;
        assert_eq!(auth.xmlns, "urn:ietf:params:xml:ns:xmpp-sasl");
        assert_eq!(auth.mechanism.to_string(), Mechanism::Plain.to_string());
        assert_eq!(auth.value, "AGp1bGlldAByMG0zMG15cjBtMzA=");
        Ok(())
    }

    #[test]
    fn test_auth_success() -> eyre::Result<()> {
        let xml = r#"<success xmlns="urn:ietf:params:xml:ns:xmpp-sasl"/>"#;
        let success = AuthSuccess::read_xml_string(xml)?;
        assert_eq!(success.xmlns, "urn:ietf:params:xml:ns:xmpp-sasl");
        Ok(())
    }

    #[test]
    fn test_plaintext_credentials() -> eyre::Result<()> {
        let credentials = PlaintextCredentials::new("jid".to_string(), "password".to_string());
        let base64 = credentials.to_base64();
        assert_eq!(base64, "amlkAHBhc3N3b3Jk");
        let credentials = PlaintextCredentials::from_base64(base64)?;
        assert_eq!(credentials.username, "jid");
        assert_eq!(credentials.password, "password");
        Ok(())
    }
}
