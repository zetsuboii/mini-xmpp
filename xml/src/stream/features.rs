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
#[derive(Debug, Clone)]
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

impl ReadXml<'_> for Mechanism {
    fn read_xml(reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        // <mechanism>
        let mechanism_start = match reader.read_event()? {
            Event::Start(tag) => tag,
            _ => eyre::bail!("invalid start tag"),
        };

        Self::read_xml_from_start(mechanism_start, reader)
    }

    fn read_xml_from_start<'a>(
        start: BytesStart<'a>,
        reader: &mut Reader<&[u8]>,
    ) -> eyre::Result<Self> {
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

#[derive(Default, Debug, Clone)]
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
    fn read_xml(reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        // <mechanisms>
        let mechanisms_start = reader.read_event()?;
        let mechanisms_start = match mechanisms_start {
            Event::Start(tag) => tag,
            _ => eyre::bail!("invalid start tag"),
        };

        Self::read_xml_from_start(mechanisms_start, reader)
    }

    fn read_xml_from_start<'a>(
        start: BytesStart<'a>,
        reader: &mut Reader<&[u8]>,
    ) -> eyre::Result<Self> {
        let xmlns = try_get_attribute(&start, "xmlns")?;
        let mut result = Self::new(xmlns);

        while let Ok(event) = reader.read_event() {
            match event {
                Event::Start(tag) => match tag.name().as_ref() {
                    // <mechanism>
                    b"mechanism" => result
                        .mechanisms
                        .push(Mechanism::read_xml_from_start(tag, reader)?),
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
#[derive(Default, Debug, Clone)]
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

impl ReadXml<'_> for StartTls {
    fn read_xml(reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        let start_tls_start = reader.read_event()?;
        let starttls_start = match start_tls_start {
            Event::Empty(tag) => tag,
            Event::Start(tag) => tag,
            _ => eyre::bail!("invalid start tag"),
        };

        Self::read_xml_from_start(starttls_start, reader)
    }

    fn read_xml_from_start<'a>(
        start: BytesStart<'a>,
        reader: &mut Reader<&[u8]>,
    ) -> eyre::Result<Self> {
        if start.name().as_ref() != b"starttls" {
            eyre::bail!("invalid tag name")
        }

        let xmlns = try_get_attribute(&start, "xmlns")?;
        let mut result = Self::new(xmlns);

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
                _ => {}
            }
        }

        Ok(result)
    }
}

//
// bind
//

/// Request to bind resource
/// Resource binding can be done by client, or it can be automatically assigned
/// by the server
#[derive(Default, Debug, Clone)]
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

impl ReadXml<'_> for Bind {
    fn read_xml(reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        // <bind>
        let bind_start = reader.read_event()?;
        let bind_start = match bind_start {
            Event::Empty(tag) => tag,
            Event::Start(tag) => tag,
            _ => eyre::bail!("invalid start tag"),
        };

        Self::read_xml_from_start(bind_start, reader)
    }

    fn read_xml_from_start<'a>(
        start: BytesStart<'a>,
        reader: &mut Reader<&[u8]>,
    ) -> eyre::Result<Self> {
        if start.name().as_ref() != b"bind" {
            eyre::bail!("invalid tag name")
        }

        let xmlns = try_get_attribute(&start, "xmlns")?;
        let mut result = Self::new(xmlns);

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
                _ => {}
            }
        }

        Ok(result)
    }
}

//
// stream:features
//

/// Stream features to negotiate after connection
#[derive(Default, Debug, Clone)]
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
    fn read_xml(reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        let features_start = match reader.read_event()? {
            Event::Empty(tag) => tag,
            Event::Start(tag) => tag,
            _ => eyre::bail!("invalid start tag"),
        };

        Self::read_xml_from_start(features_start, reader)
    }

    fn read_xml_from_start<'a>(
        start: BytesStart<'a>,
        reader: &mut Reader<&[u8]>,
    ) -> eyre::Result<Self> {
        if start.name().as_ref() != b"stream:features" {
            eyre::bail!("invalid tag name")
        }

        let mut result = Self::new();

        while let Ok(event) = reader.read_event() {
            match event {
                Event::Empty(tag) => match tag.name().as_ref() {
                    b"starttls" => {
                        if result.start_tls.is_some() {
                            eyre::bail!("multiple starttls tags")
                        }
                        result.start_tls = Some(StartTls::read_xml_from_start(tag, reader)?)
                    }
                    b"bind" => {
                        if result.bind.is_some() {
                            eyre::bail!("multiple bind tags")
                        }
                        result.bind = Some(Bind::read_xml_from_start(tag, reader)?)
                    }
                    _ => eyre::bail!("invalid empty tag"),
                },
                Event::Start(tag) => match tag.name().as_ref() {
                    b"starttls" => {
                        if result.start_tls.is_some() {
                            eyre::bail!("multiple starttls tags")
                        }
                        result.start_tls = Some(StartTls::read_xml_from_start(tag, reader)?)
                    }
                    b"bind" => {
                        if result.bind.is_some() {
                            eyre::bail!("multiple bind tags")
                        }
                        result.bind = Some(Bind::read_xml_from_start(tag, reader)?)
                    }
                    b"mechanisms" => {
                        if result.mechanisms.is_some() {
                            eyre::bail!("multiple mechanisms tags")
                        }
                        result.mechanisms = Some(Mechanisms::read_xml_from_start(tag, reader)?)
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
    use crate::utils::Collect;

    use super::*;

    fn mechanism_equals(a: &Mechanism, b: &Mechanism) -> bool {
        match (a, b) {
            (Mechanism::Plain, Mechanism::Plain) => true,
        }
    }

    fn starttls_equals(a: &StartTls, b: &StartTls) -> bool {
        a.xmlns == b.xmlns && a.required == b.required
    }

    fn bind_equals(a: &Bind, b: &Bind) -> bool {
        a.xmlns == b.xmlns && a.resource == b.resource
    }

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

        let mut writer = Writer::new(Cursor::new(Vec::new()));
        mechanisms.write_xml(&mut writer).unwrap();
        let written = writer.collect();
        assert_eq!(
            written,
            "<mechanisms xmlns=\"urn:ietf:params:xml:ns:xmpp-sasl\"><mechanism>PLAIN</mechanism></mechanisms>"
        );

        let mut reader = Reader::from_str(written.as_str());
        reader.trim_text(true);

        let read = Mechanisms::read_xml(&mut reader).unwrap();
        assert_eq!(mechanisms.xmlns, read.xmlns);
        for (mechanism, read_mechanism) in mechanisms.mechanisms.iter().zip(read.mechanisms.iter())
        {
            assert!(mechanism_equals(mechanism, read_mechanism));
        }
    }

    #[test]
    fn test_starttls() {
        let starttls = StartTls {
            xmlns: "urn:ietf:params:xml:ns:xmpp-tls".to_string(),
            required: true,
        };

        let mut writer = Writer::new(Cursor::new(Vec::new()));
        starttls.write_xml(&mut writer).unwrap();
        let written = writer.collect();
        assert_eq!(
            written,
            "<starttls xmlns=\"urn:ietf:params:xml:ns:xmpp-tls\"><required/></starttls>"
        );

        let mut reader = Reader::from_str(written.as_str());
        reader.trim_text(true);

        let read = StartTls::read_xml(&mut reader).unwrap();
        assert!(starttls_equals(&starttls, &read));
    }

    #[test]
    fn test_bind() {
        let bind = Bind {
            xmlns: "urn:ietf:params:xml:ns:xmpp-bind".to_string(),
            resource: Some("resource".to_string()),
        };

        let mut writer = Writer::new(Cursor::new(Vec::new()));
        bind.write_xml(&mut writer).unwrap();
        let written = writer.collect();
        assert_eq!(
            written,
            "<bind xmlns=\"urn:ietf:params:xml:ns:xmpp-bind\"><resource>resource</resource></bind>"
        );

        let mut reader = Reader::from_str(written.as_str());
        reader.trim_text(true);

        let read = Bind::read_xml(&mut reader).unwrap();
        assert!(bind_equals(&bind, &read));
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

        let mut writer = Writer::new(Cursor::new(Vec::new()));
        features.write_xml(&mut writer).unwrap();
        let written = writer.collect();
        assert_eq!(
            written,
            [
                "<stream:features>",
                "<starttls xmlns=\"urn:ietf:params:xml:ns:xmpp-tls\"><required/></starttls>",
                "<mechanisms xmlns=\"urn:ietf:params:xml:ns:xmpp-sasl\"><mechanism>PLAIN</mechanism></mechanisms>",
                "<bind xmlns=\"urn:ietf:params:xml:ns:xmpp-bind\"><resource>resource</resource></bind>",
                "</stream:features>"
            ].concat()
        );

        let mut reader = Reader::from_str(written.as_str());
        reader.trim_text(true);

        let read = Features::read_xml(&mut reader).unwrap();
        assert!(starttls_equals(
            features.start_tls.as_ref().unwrap(),
            read.start_tls.as_ref().unwrap()
        ));
        for (mechanism, read_mechanism) in features
            .mechanisms
            .as_ref()
            .unwrap()
            .mechanisms
            .iter()
            .zip(read.mechanisms.as_ref().unwrap().mechanisms.iter())
        {
            assert!(mechanism_equals(mechanism, read_mechanism));
        }
        assert!(bind_equals(
            features.bind.as_ref().unwrap(),
            read.bind.as_ref().unwrap()
        ));
    }

    #[test]
    fn test_features_empty() {
        let features = Features::new();

        let mut writer = Writer::new(Cursor::new(Vec::new()));
        features.write_xml(&mut writer).unwrap();
        let written = writer.collect();
        assert_eq!(written, "<stream:features></stream:features>");

        let mut reader = Reader::from_str(written.as_str());
        reader.trim_text(true);

        let read = Features::read_xml(&mut reader).unwrap();
        assert!(features.is_empty());
        assert!(read.is_empty());
    }
}
