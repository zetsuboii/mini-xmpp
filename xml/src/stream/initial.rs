use std::io::Cursor;
use color_eyre::eyre;

use quick_xml::{
    events::{BytesStart, Event}, Reader, Writer
};

use crate::{from_xml::{FromXml, ToXml}, utils::Collect};

/// Initial header to start XMPP connection
/// 
/// https://www.rfc-editor.org/rfc/rfc6120.html#section-4.2
#[derive(Debug, Default)]
pub struct InitialHeader {
    pub id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub version: Option<String>,
    pub xml_lang: Option<String>,
    pub xmlns: Option<String>,
    pub xmlns_stream: Option<String>,
}

impl ToXml for InitialHeader {
    /// Custom XML serializer for StreamHeader
    /// 
    /// This is requrired as quick_xml with serde doesn't allow namespaces and
    /// parsing starting/ending headers by themselves
    fn to_xml(&self) -> String {
        let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));

        let mut stream_header = BytesStart::new("stream:stream");
        if let Some(id) = &self.id {
            stream_header.push_attribute(("id", id.as_str()));
        }
        if let Some(id) = &self.id {
            stream_header.push_attribute(("id", id.as_str()));
        }
        if let Some(from) = &self.from {
            stream_header.push_attribute(("from", from.as_str()));
        }
        if let Some(to) = &self.id {
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

        writer.write_event(Event::Start(stream_header)).unwrap();
        writer.collect()
    }
}

impl<T: AsRef<str>> FromXml<T> for InitialHeader {
    type Out = Self;

    /// Custom XML deserializer for StreamHeader
    /// 
    /// This is requrired as quick_xml with serde doesn't allow namespaces and
    /// parsing starting/ending headers by themselves
    fn from_xml(xml: T) -> eyre::Result<Self> {
        let mut reader = Reader::from_str(xml.as_ref());
        let mut result = Self::new();

        loop {
            if let Ok(event) = reader.read_event() {
                match event {
                    Event::Eof => break,
                    Event::Start(e) => {
                        let name = e.name();
                        if name.as_ref() != b"stream:stream" {
                            eyre::bail!(
                                "expected stream:stream got {:?}",
                                std::str::from_utf8(name.as_ref())
                            );
                        }

                        e.attributes().for_each(|attr| {
                            if let Ok(attr) = attr {
                                let key = attr.key.0;
                                println!("{:?}", std::str::from_utf8(key.as_ref()));
                                let value = std::str::from_utf8(&attr.value).unwrap().to_string();

                                match key {
                                    b"id" => result.id = Some(value),
                                    b"from" => result.from = Some(value),
                                    b"to" => result.to = Some(value),
                                    b"version" => result.version = Some(value),
                                    b"xml:lang" => result.xml_lang = Some(value),
                                    b"xmlns" => result.xmlns = Some(value),
                                    b"xmlns:stream" =>result.xmlns_stream = Some(value),
                                    _ => {}
                                }
                            }
                        })
                    }
                    _ => {}
                }
            }
        }

        return Ok(result);
    }
}

impl InitialHeader {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize() {
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

        // let stream_header: StreamHeader = (raw.as_ref()).unwrap();
        // let stream_header = stream_header.unescaped();
        let stream_header = InitialHeader::from_xml(raw).unwrap();

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
