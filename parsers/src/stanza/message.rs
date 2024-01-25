use std::io::Cursor;

use color_eyre::eyre;
use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    name::QName,
    Writer,
};

use crate::{
    from_xml::{ReadXml, WriteXml},
    utils::try_get_attribute,
};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub body: Option<String>,
    pub xml_lang: Option<String>,
}

impl Message {
    pub fn new() -> Self {
        Default::default()
    }
}

impl ReadXml<'_> for Message {
    fn read_xml<'a>(root: Event<'a>, reader: &mut quick_xml::Reader<&[u8]>) -> eyre::Result<Self> {
        let start = match root {
            Event::Start(tag) => tag,
            _ => eyre::bail!("invalid start tag"),
        };
        if start.name().as_ref() != b"message" {
            eyre::bail!("invalid tag name")
        }

        let mut result = Self::new();

        // <message id from to xml:lang>
        result.id = try_get_attribute(&start, "id").ok();
        result.from = try_get_attribute(&start, "from").ok();
        result.to = try_get_attribute(&start, "to").ok();
        result.xml_lang = try_get_attribute(&start, "xml:lang").ok();

        match reader.read_event()? {
            // <body>
            Event::Start(tag) => {
                if tag.name().as_ref() != b"body" {
                    eyre::bail!("invalid start tag")
                }
                // { body }
                // </body>
                result.body = reader
                    .read_text(QName(b"body"))
                    .map(|body| body.to_string())
                    .ok();
            }
            _ => {}
        }

        Ok(result)
    }
}

impl WriteXml for Message {
    fn write_xml(&self, writer: &mut Writer<Cursor<Vec<u8>>>) -> eyre::Result<()> {
        // <message from={...} to={...}>
        let mut message_start = BytesStart::new("message");
        if let Some(id) = &self.id {
            message_start.push_attribute(("id", id.as_ref()));
        }
        if let Some(from) = &self.from {
            message_start.push_attribute(("from", from.as_ref()));
        }
        if let Some(to) = &self.to {
            message_start.push_attribute(("to", to.as_ref()));
        }
        if let Some(xml_lang) = &self.xml_lang {
            message_start.push_attribute(("xml:lang", xml_lang.as_ref()));
        }

        writer.write_event(Event::Start(message_start)).unwrap();

        if let Some(body) = &self.body {
            // <body>
            writer
                .write_event(Event::Start(BytesStart::new("body")))
                .unwrap();
            // {...}
            writer
                .write_event(Event::Text(BytesText::new(body.as_ref())))
                .unwrap();
            // </body>
            writer
                .write_event(Event::End(BytesEnd::new("body")))
                .unwrap();
        }

        // </message>
        writer.write_event(Event::End(BytesEnd::new("message")))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::from_xml::{ReadXmlString, WriteXmlString};

    use super::*;

    #[test]
    fn test_message_empty() {
        let message: Message = Message::new();

        let serialized = message.write_xml_string().unwrap();
        let expected = r#"<message></message>"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_message_full() {
        let message = Message {
            id: Some("123".to_string()),
            from: Some("alice@mail.com".to_string()),
            to: Some("bob@mail.com".to_string()),
            body: Some("Hello, world!".to_string()),
            xml_lang: Some("en".to_string()),
        };

        let serialized = message.write_xml_string().unwrap();
        let expected = [
            "<message ",
            "id=\"123\" ",
            "from=\"alice@mail.com\" ",
            "to=\"bob@mail.com\" ",
            "xml:lang=\"en\">",
            "<body>Hello, world!</body>",
            "</message>",
        ]
        .concat();
        assert_eq!(serialized, expected);

        let deserialized: Message = Message::read_xml_string(serialized.as_str()).unwrap();
        assert_eq!(deserialized, message);
    }
}
