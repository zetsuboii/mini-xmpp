use serde::{Deserialize, Serialize};

use crate::utils::unescape_colon;

//
// Stream Header
//

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename = "stream_COLON_stream")]
pub struct StreamHeader {
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "@from")]
    pub from: Option<String>,
    #[serde(rename = "@to")]
    pub to: Option<String>,
    #[serde(rename = "@version")]
    pub version: Option<String>,
    #[serde(rename = "@xml_COLON_lang")]
    pub xml_lang: Option<String>,
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,
    #[serde(rename = "@xmlns_COLON_stream")]
    pub xmlns_stream: Option<String>,
}

impl StreamHeader {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn unescaped(mut self) -> Self {
        self.id = self.id.map(unescape_colon);
        self.from = self.from.map(unescape_colon);
        self.to = self.to.map(unescape_colon);
        self.version = self.version.map(unescape_colon);
        self.xml_lang = self.xml_lang.map(unescape_colon);
        self.xmlns = self.xmlns.map(unescape_colon);
        self.xmlns_stream = self.xmlns_stream.map(unescape_colon);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::escape_colon;

    use super::*;
    use quick_xml::de::from_str;

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
            xmlns:stream='http://etherx.jabber.org/streams'/>
        "#;

        let raw = escape_colon(raw);
        let stream_header: StreamHeader = from_str(raw.as_ref()).unwrap();
        let stream_header = stream_header.unescaped();

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

