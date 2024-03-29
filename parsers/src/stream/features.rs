//! Stream features and related structs

use color_eyre::eyre;
use std::io::Cursor;

use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Reader, Writer,
};

use crate::{
    empty::IsEmpty,
    from_xml::{ReadXml, WriteXml},
    utils::try_get_attribute,
};

//
// mechanisms
//

/// Mechanisms used in the communication, includes hashes and authentication
/// methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mechanism {
    /// Plaintext authentication mechanism
    Plain,
}

impl ToString for Mechanism {
    fn to_string(&self) -> String {
        match self {
            Mechanism::Plain => "PLAIN",
        }
        .to_string()
    }
}

impl TryFrom<&str> for Mechanism {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "PLAIN" => Ok(Self::Plain),
            _ => eyre::bail!("invalid mechanism"),
        }
    }
}

impl ReadXml<'_> for Mechanism {
    fn read_xml<'a>(event: Event<'a>, reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        // <mechanism>
        let start = match event {
            Event::Start(tag) => tag,
            _ => eyre::bail!("invalid start tag"),
        };
        if start.name().as_ref() != b"mechanism" {
            eyre::bail!("invalid tag name")
        }

        // { mechanism }
        let text = match reader.read_event()? {
            Event::Text(text) => String::from_utf8(text.to_vec())?,
            _ => eyre::bail!("invalid text"),
        };
        let mechanism = match text.as_str() {
            "PLAIN" => Self::Plain,
            _ => eyre::bail!("invalid mechanism"),
        };

        // </mechanism>
        match reader.read_event()? {
            Event::End(tag) => match tag.name().as_ref() {
                b"mechanism" => {}
                _ => eyre::bail!("invalid end tag"),
            },
            _ => eyre::bail!("invalid end tag"),
        }

        Ok(mechanism)
    }
}

impl WriteXml for Mechanism {
    fn write_xml(&self, writer: &mut Writer<Cursor<Vec<u8>>>) -> eyre::Result<()> {
        // <mechanism>
        writer.write_event(Event::Start(BytesStart::new("mechanism")))?;
        // { mechanism }
        writer.write_event(Event::Text(BytesText::new(&self.to_string())))?;
        // </mechanism>
        writer.write_event(Event::End(BytesEnd::new("mechanism")))?;

        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Mechanisms {
    pub xmlns: String,
    pub mechanisms: Vec<Mechanism>,
}

impl Mechanisms {
    fn new(xmlns: String) -> Self {
        Self {
            xmlns,
            ..Default::default()
        }
    }
}

impl ReadXml<'_> for Mechanisms {
    // fn read_xml(reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
    //     // <mechanisms>
    //     let mechanisms_start = reader.read_event()?;
    //     let mechanisms_start = match mechanisms_start {
    //         Event::Start(tag) => tag,
    //         _ => eyre::bail!("invalid start tag"),
    //     };

    //     Self::read_xml_from_start(mechanisms_start, reader)
    // }

    fn read_xml<'a>(root: Event<'a>, reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        let start = match root {
            Event::Start(tag) => tag,
            _ => eyre::bail!("invalid start tag"),
        };
        let xmlns = try_get_attribute(&start, "xmlns")?;
        let mut result = Self::new(xmlns);

        while let Ok(event) = reader.read_event() {
            match event {
                Event::Start(ref tag) => match tag.name().as_ref() {
                    // <mechanism>
                    b"mechanism" => result.mechanisms.push(Mechanism::read_xml(event, reader)?),
                    _ => eyre::bail!("invalid start tag"),
                },
                Event::End(tag) => match tag.name().as_ref() {
                    // </mechanisms>
                    b"mechanisms" => break,
                    _ => eyre::bail!("invalid end tag"),
                },
                _ => {}
            }
        }

        Ok(result)
    }
}

impl WriteXml for Mechanisms {
    fn write_xml(&self, writer: &mut Writer<Cursor<Vec<u8>>>) -> eyre::Result<()> {
        // <mechanisms xmlns>
        let mut mechanisms_start = BytesStart::new("mechanisms");
        mechanisms_start.push_attribute(("xmlns", self.xmlns.as_ref()));
        writer.write_event(Event::Start(mechanisms_start))?;

        for mechanism in self.mechanisms.iter() {
            // <mechanism>
            writer.write_event(Event::Start(BytesStart::new("mechanism")))?;
            // { mechanism }
            writer.write_event(Event::Text(BytesText::new(&mechanism.to_string())))?;
            // </mechanism>
            writer.write_event(Event::End(BytesEnd::new("mechanism")))?;
        }

        // </mechanisms>
        writer.write_event(Event::End(BytesEnd::new("mechanisms")))?;

        Ok(())
    }
}

//
// starttls
//

/// Request to start TLS connection
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct StartTls {
    pub xmlns: String,
    /// If TLS connection is required
    pub required: bool,
}

impl StartTls {
    pub fn new(xmlns: String) -> Self {
        Self {
            xmlns,
            ..Default::default()
        }
    }
}

impl IsEmpty for StartTls {
    fn is_empty(&self) -> bool {
        self.required
    }
}

impl ReadXml<'_> for StartTls {
    fn read_xml<'a>(root: Event<'a>, reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        let (start, empty) = match root {
            Event::Empty(tag) => (tag, true),
            Event::Start(tag) => (tag, false),
            _ => eyre::bail!("invalid start tag"),
        };
        if start.name().as_ref() != b"starttls" {
            eyre::bail!("invalid tag name")
        }

        let xmlns = try_get_attribute(&start, "xmlns")?;
        let mut result = Self::new(xmlns);

        if empty {
            return Ok(result);
        }

        while let Ok(event) = reader.read_event() {
            match event {
                Event::Empty(tag) => match tag.name().as_ref() {
                    b"required" => result.required = true,
                    _ => eyre::bail!("invalid empty tag"),
                },
                Event::End(tag) => match tag.name().as_ref() {
                    b"starttls" => break,
                    _ => eyre::bail!("invalid end tag"),
                },
                Event::Eof => eyre::bail!("unexpected EOF"),
                _ => {}
            }
        }

        Ok(result)
    }
}

impl WriteXml for StartTls {
    fn write_xml(&self, writer: &mut Writer<Cursor<Vec<u8>>>) -> eyre::Result<()> {
        let mut starttls_start = BytesStart::new("starttls");
        starttls_start.push_attribute(("xmlns", self.xmlns.as_ref()));

        if self.required {
            // <starttls xmlns>
            writer.write_event(Event::Start(starttls_start))?;
            // <required/>
            writer.write_event(Event::Empty(BytesStart::new("required")))?;
            // </starttls>
            writer.write_event(Event::End(BytesEnd::new("starttls")))?;
        } else {
            // <starttls xmlns/>
            writer.write_event(Event::Empty(starttls_start)).unwrap();
        }

        Ok(())
    }
}

//
// starttls responses
//

/// Request to start TLS connection
#[derive(Debug, Clone)]
pub struct StartTlsResponse {
    pub xmlns: String,
    /// Result of the TLS connection
    pub result: StartTlsResult,
}

#[derive(Debug, Clone)]
pub enum StartTlsResult {
    /// TLS connection succeeded
    Proceed,
    /// TLS connection failed
    Failure,
}

impl ReadXml<'_> for StartTlsResponse {
    fn read_xml<'a>(root: Event<'a>, _reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        let start = match root {
            Event::Empty(tag) => tag,
            _ => eyre::bail!("invalid start tag"),
        };

        let xmlns = try_get_attribute(&start, "xmlns")?;
        let result = match start.name().as_ref() {
            b"proceed" => StartTlsResult::Proceed,
            b"failure" => StartTlsResult::Failure,
            _ => eyre::bail!("invalid tag name"),
        };
        Ok(Self { xmlns, result })
    }
}

impl WriteXml for StartTlsResponse {
    fn write_xml(&self, writer: &mut Writer<Cursor<Vec<u8>>>) -> eyre::Result<()> {
        let mut result_start = match self.result {
            StartTlsResult::Proceed => {
                // // <proceed/>
                BytesStart::new("proceed")
            }
            StartTlsResult::Failure => {
                // <failure/>
                BytesStart::new("failure")
            }
        };
        result_start.push_attribute(("xmlns", self.xmlns.as_ref()));
        writer.write_event(Event::Empty(result_start))?;
        Ok(())
    }
}

//
// bind
//

/// Request to bind resource
/// Resource binding can be done by client, or it can be automatically assigned
/// by the server
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Bind {
    pub xmlns: String,
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
        self.resource.is_none()
    }
}

impl ReadXml<'_> for Bind {
    fn read_xml<'a>(root: Event<'a>, reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        let (start, empty) = match root {
            Event::Empty(tag) => (tag, true),
            Event::Start(tag) => (tag, false),
            _ => eyre::bail!("invalid start tag"),
        };
        if start.name().as_ref() != b"bind" {
            eyre::bail!("invalid tag name")
        }

        let xmlns = try_get_attribute(&start, "xmlns")?;
        let mut result = Self::new(xmlns);

        if empty {
            return Ok(result);
        }

        while let Ok(event) = reader.read_event() {
            match event {
                Event::Start(tag) => match tag.name().as_ref() {
                    // <resource>
                    b"resource" => {
                        // { resource }
                        let resource_text = match reader.read_event()? {
                            Event::Text(text) => text.to_vec(),
                            _ => eyre::bail!("invalid resource content"),
                        };
                        result.resource = Some(String::from_utf8(resource_text)?);

                        // </resource>
                        match reader.read_event()? {
                            Event::End(tag) => match tag.name().as_ref() {
                                b"resource" => {}
                                _ => eyre::bail!("invalid end tag"),
                            },
                            _ => eyre::bail!("invalid resource end"),
                        }
                    }
                    _ => eyre::bail!("invalid bind content"),
                },
                Event::End(tag) => match tag.name().as_ref() {
                    // </bind>
                    b"bind" => break,
                    _ => eyre::bail!("invalid end tag"),
                },
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

        if let Some(text) = &self.resource {
            // <bind>
            writer.write_event(Event::Start(bind_start))?;
            // <resource>
            writer.write_event(Event::Start(BytesStart::new("resource")))?;
            // { resource }
            writer.write_event(Event::Text(BytesText::new(text)))?;
            // </resource>
            writer.write_event(Event::End(BytesEnd::new("resource")))?;
            // </bind>
            writer.write_event(Event::End(BytesEnd::new("bind")))?;
        } else {
            // <bind/>
            writer.write_event(Event::Empty(bind_start))?;
        }

        Ok(())
    }
}

//
// stream:features
//

/// Stream features to negotiate after connection
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Features {
    pub start_tls: Option<StartTls>,
    pub mechanisms: Option<Mechanisms>,
    pub bind: Option<Bind>,
}

impl Features {
    pub fn new() -> Self {
        Default::default()
    }
}

impl IsEmpty for Features {
    fn is_empty(&self) -> bool {
        self.start_tls.is_none() && self.mechanisms.is_none() && self.bind.is_none()
    }
}

impl ReadXml<'_> for Features {
    fn read_xml<'a>(root: Event<'a>, reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        let start = match root {
            Event::Empty(tag) => tag,
            Event::Start(tag) => tag,
            _ => eyre::bail!("invalid start tag"),
        };
        if start.name().as_ref() != b"stream:features" {
            eyre::bail!("invalid tag name")
        }

        let mut result = Self::new();

        while let Ok(event) = reader.read_event() {
            match event {
                Event::Empty(ref tag) => match tag.name().as_ref() {
                    b"starttls" => {
                        if result.start_tls.is_some() {
                            eyre::bail!("multiple starttls tags")
                        }
                        result.start_tls = Some(StartTls::read_xml(event, reader)?)
                    }
                    b"bind" => {
                        if result.bind.is_some() {
                            eyre::bail!("multiple bind tags")
                        }
                        result.bind = Some(Bind::read_xml(event, reader)?)
                    }
                    _ => eyre::bail!("invalid empty tag"),
                },
                Event::Start(ref tag) => match tag.name().as_ref() {
                    b"starttls" => {
                        if result.start_tls.is_some() {
                            eyre::bail!("multiple starttls tags")
                        }
                        result.start_tls = Some(StartTls::read_xml(event, reader)?)
                    }
                    b"bind" => {
                        if result.bind.is_some() {
                            eyre::bail!("multiple bind tags")
                        }
                        result.bind = Some(Bind::read_xml(event, reader)?)
                    }
                    b"mechanisms" => {
                        if result.mechanisms.is_some() {
                            eyre::bail!("multiple mechanisms tags")
                        }
                        result.mechanisms = Some(Mechanisms::read_xml(event, reader)?)
                    }
                    _ => eyre::bail!("invalid start tag"),
                },
                Event::End(tag) => match tag.name().as_ref() {
                    b"stream:features" => break,
                    _ => eyre::bail!(
                        "invalid end tag {}",
                        String::from_utf8_lossy(tag.name().as_ref())
                    ),
                },
                Event::Eof => eyre::bail!("unexpected EOF"),
                _ => {}
            }
        }

        Ok(result)
    }
}

impl WriteXml for Features {
    fn write_xml(&self, writer: &mut Writer<Cursor<Vec<u8>>>) -> eyre::Result<()> {
        writer.write_event(Event::Start(BytesStart::new("stream:features")))?;

        if let Some(start_tls) = &self.start_tls {
            start_tls.write_xml(writer)?;
        }
        if let Some(mechanisms) = &self.mechanisms {
            mechanisms.write_xml(writer)?;
        }
        if let Some(bind) = &self.bind {
            bind.write_xml(writer)?;
        }

        writer.write_event(Event::End(BytesEnd::new("stream:features")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::from_xml::{ReadXmlString, WriteXmlString};

    use super::*;

    #[test]
    fn test_mechanism() {
        let mechanism = Mechanism::Plain;
        assert_eq!(mechanism.to_string(), "PLAIN");
    }

    #[test]
    fn test_mechanisms() {
        let mechanisms = Mechanisms {
            xmlns: "urn:ietf:params:xml:ns:xmpp-sasl".to_string(),
            mechanisms: vec![Mechanism::Plain],
        };

        let serialized = mechanisms.write_xml_string().unwrap();
        assert_eq!(
            serialized,
            "<mechanisms xmlns=\"urn:ietf:params:xml:ns:xmpp-sasl\"><mechanism>PLAIN</mechanism></mechanisms>"
        );

        let deserialized = Mechanisms::read_xml_string(&serialized).unwrap();
        assert_eq!(
            deserialized,
            Mechanisms {
                xmlns: "urn:ietf:params:xml:ns:xmpp-sasl".to_string(),
                mechanisms: vec![Mechanism::Plain],
            }
        );
    }

    #[test]
    fn test_starttls() {
        let starttls = StartTls {
            xmlns: "urn:ietf:params:xml:ns:xmpp-tls".to_string(),
            required: true,
        };

        let serialized = starttls.write_xml_string().unwrap();
        assert_eq!(
            serialized,
            "<starttls xmlns=\"urn:ietf:params:xml:ns:xmpp-tls\"><required/></starttls>"
        );

        let deserialized = StartTls::read_xml_string(&serialized).unwrap();
        assert_eq!(
            deserialized,
            StartTls {
                xmlns: "urn:ietf:params:xml:ns:xmpp-tls".to_string(),
                required: true,
            }
        )
    }

    #[test]
    fn test_bind() {
        let bind = Bind {
            xmlns: "urn:ietf:params:xml:ns:xmpp-bind".to_string(),
            resource: Some("resource".to_string()),
        };

        let serialized = bind.write_xml_string().unwrap();
        assert_eq!(
            serialized,
            [
                "<bind xmlns=\"urn:ietf:params:xml:ns:xmpp-bind\">",
                "<resource>resource</resource>",
                "</bind>"
            ]
            .concat()
        );

        let deserialized = Bind::read_xml_string(&serialized).unwrap();
        assert_eq!(deserialized, Bind {
            xmlns: "urn:ietf:params:xml:ns:xmpp-bind".to_string(),
            resource: Some("resource".to_string()),
        })
    }

    #[test]
    fn test_features() {
        let features = Features {
            start_tls: Some(StartTls {
                xmlns: "urn:ietf:params:xml:ns:xmpp-tls".to_string(),
                required: true,
            }),
            mechanisms: Some(Mechanisms {
                xmlns: "urn:ietf:params:xml:ns:xmpp-sasl".to_string(),
                mechanisms: vec![Mechanism::Plain],
            }),
            bind: Some(Bind {
                xmlns: "urn:ietf:params:xml:ns:xmpp-bind".to_string(),
                resource: Some("resource".to_string()),
            }),
        };

        let serialized = features.write_xml_string().unwrap();
        assert_eq!(
            serialized,
            [
                "<stream:features>",
                "<starttls xmlns=\"urn:ietf:params:xml:ns:xmpp-tls\"><required/></starttls>",
                "<mechanisms xmlns=\"urn:ietf:params:xml:ns:xmpp-sasl\"><mechanism>PLAIN</mechanism></mechanisms>",
                "<bind xmlns=\"urn:ietf:params:xml:ns:xmpp-bind\"><resource>resource</resource></bind>",
                "</stream:features>"
            ].concat()
        );

        let deserialized = Features::read_xml_string(&serialized).unwrap();
        assert_eq!(deserialized, Features {
            start_tls: Some(StartTls {
                xmlns: "urn:ietf:params:xml:ns:xmpp-tls".to_string(),
                required: true,
            }),
            mechanisms: Some(Mechanisms {
                xmlns: "urn:ietf:params:xml:ns:xmpp-sasl".to_string(),
                mechanisms: vec![Mechanism::Plain],
            }),
            bind: Some(Bind {
                xmlns: "urn:ietf:params:xml:ns:xmpp-bind".to_string(),
                resource: Some("resource".to_string()),
            }),
        })
    }

    #[test]
    fn test_features_empty() {
        let features = Features::new();

        let serialized = features.write_xml_string().unwrap();
        assert_eq!(serialized, "<stream:features></stream:features>");

        let read = Features::read_xml_string(&serialized).unwrap();
        assert!(features.is_empty());
        assert!(read.is_empty());
    }
}
