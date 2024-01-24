//! Initial header to start XMPP connection

use color_eyre::eyre;
use std::io::Cursor;

use quick_xml::{
    events::{BytesStart, Event},
    Reader, Writer,
};

use crate::from_xml::{ReadXml, WriteXml};

/// Initial header to start XMPP connection
///
/// https://www.rfc-editor.org/rfc/rfc6120.html#section-4.2
#[derive(Default, Debug)]
pub struct InitialHeader {
    pub id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub version: Option<String>,
    pub xml_lang: Option<String>,
    pub xmlns: Option<String>,
    pub xmlns_stream: Option<String>,
}

impl InitialHeader {
    pub fn new() -> Self {
        Default::default()
    }
}

impl ReadXml<'_> for InitialHeader {
    fn read_xml(reader: &mut Reader<&[u8]>) -> eyre::Result<Self> {
        // <stream:stream>
        let stream_start = match reader.read_event()? {
            Event::Start(tag) => tag,
            _ => eyre::bail!("invalid start tag"),
        };

        Self::read_xml_from_start(stream_start, reader)
    }

    fn read_xml_from_start<'a>(
        start: BytesStart<'a>,
        _reader: &mut Reader<&[u8]>,
    ) -> eyre::Result<Self> {
        if start.name().as_ref() != b"stream:stream" {
            eyre::bail!("invalid tag name")
        }

        let mut result = Self::new();
        start.attributes().for_each(|attr| {
            if let Ok(attr) = attr {
                let key = attr.key.0;
                let value = std::str::from_utf8(&attr.value).unwrap().to_string();

                match key {
                    b"id" => result.id = Some(value),
                    b"from" => result.from = Some(value),
                    b"to" => result.to = Some(value),
                    b"version" => result.version = Some(value),
                    b"xml:lang" => result.xml_lang = Some(value),
                    b"xmlns" => result.xmlns = Some(value),
                    b"xmlns:stream" => result.xmlns_stream = Some(value),
                    _ => {}
                }
            }
        });

        Ok(result)
    }
}

impl WriteXml for InitialHeader {
    fn write_xml(&self, writer: &mut Writer<Cursor<Vec<u8>>>) -> eyre::Result<()> {
        let mut stream_header = BytesStart::new("stream:stream");
        if let Some(id) = &self.id {
            stream_header.push_attribute(("id", id.as_str()));
        }
        if let Some(from) = &self.from {
            stream_header.push_attribute(("from", from.as_str()));
        }
        if let Some(to) = &self.to {
            stream_header.push_attribute(("to", to.as_str()));
        }
        if let Some(version) = &self.version {
            stream_header.push_attribute(("version", version.as_str()));
        }
        if let Some(xml_lang) = &self.xml_lang {
            stream_header.push_attribute(("xml:lang", xml_lang.as_str()));
        }
        if let Some(xmlns) = &self.xmlns {
            stream_header.push_attribute(("xmlns", xmlns.as_str()));
        }
        if let Some(xmlns_stream) = &self.xmlns_stream {
            stream_header.push_attribute(("xmlns:stream", xmlns_stream.as_str()));
        }

        writer.write_event(Event::Start(stream_header))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::from_xml::{ReadXmlString, WriteXmlString};

    use super::*;

    #[test]
    fn test_serialize() {
        let stream_header = InitialHeader {
            id: Some("++TR84Sm6A3hnt3Q065SnAbbk3Y=".to_string()),
            from: Some("im.example.com".to_string()),
            to: Some("juliet@im.example.com".to_string()),
            version: Some("1.0".to_string()),
            xml_lang: Some("en".to_string()),
            xmlns: Some("jabber:client".to_string()),
            xmlns_stream: Some("http://etherx.jabber.org/streams".to_string()),
        };

        let expected = [
            "<stream:stream ",
            "id=\"++TR84Sm6A3hnt3Q065SnAbbk3Y=\" ",
            "from=\"im.example.com\" ",
            "to=\"juliet@im.example.com\" ",
            "version=\"1.0\" ",
            "xml:lang=\"en\" ",
            "xmlns=\"jabber:client\" ",
            "xmlns:stream=\"http://etherx.jabber.org/streams\">",
        ]
        .concat();

        let serialized = stream_header.write_xml_string().unwrap();
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_deserialize() {
        let raw = r#"
        <stream:stream
            from='im.example.com'
            id='++TR84Sm6A3hnt3Q065SnAbbk3Y='
            to='juliet@im.example.com'
            version='1.0'
            xml:lang='en'
            xmlns='jabber:client'
            xmlns:stream='http://etherx.jabber.org/streams'>
        "#;

        let stream_header = InitialHeader::read_xml_string(raw).unwrap();

        assert_eq!(
            stream_header.id,
            Some("++TR84Sm6A3hnt3Q065SnAbbk3Y=".to_string())
        );
        assert_eq!(stream_header.from, Some("im.example.com".to_string()));
        assert_eq!(stream_header.to, Some("juliet@im.example.com".to_string()));
        assert_eq!(stream_header.version, Some("1.0".to_string()));
        assert_eq!(stream_header.xml_lang, Some("en".to_string()));
        assert_eq!(stream_header.xmlns, Some("jabber:client".to_string()));
        assert_eq!(
            stream_header.xmlns_stream,
            Some("http://etherx.jabber.org/streams".to_string())
        );
    }
}
