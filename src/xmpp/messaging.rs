use std::io::Cursor;

use color_eyre::eyre;
use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Reader, Writer,
};

use crate::{Collect, XmlCustomDeserialize, XmlCustomSerialize};

pub enum Stanza {
    Message(StanzaMessage),
    Presence,
    Iq,
}

impl XmlCustomSerialize for Stanza {
    fn into_string(&self) -> String {
        let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));

        match self {
            Self::Message(message) => {
                // <message to={message.to}>
                let mut message_header = BytesStart::new("message");
                message_header.push_attribute(("to", message.to.as_ref()));
                writer.write_event(Event::Start(message_header)).unwrap();
                // <body>
                writer
                    .write_event(Event::Start(BytesStart::new("body")))
                    .unwrap();
                // { text }
                writer
                    .write_event(Event::Text(BytesText::new(&message.body.as_ref())))
                    .unwrap();
                // </body>
                writer
                    .write_event(Event::End(BytesEnd::new("body")))
                    .unwrap();
                // </message>
                writer
                    .write_event(Event::End(BytesEnd::new("message")))
                    .unwrap();
            }
            Self::Iq => {
                todo!()
            }
            Self::Presence => {
                todo!()
            }
        }

        writer.collect()
    }
}

impl XmlCustomDeserialize for Stanza {
    fn from_string(value: &str) -> color_eyre::eyre::Result<Self> {
        let mut reader = Reader::from_str(value);

        if let Ok(Event::Start(e)) = reader.read_event() {
            match e.name().as_ref() {
                b"message" => {
                    // attribute `to`
                    let to = e
                        .try_get_attribute("to")
                        .unwrap()
                        .map(|attr| String::from_utf8(attr.value.to_vec()).ok())
                        .flatten()
                        .expect("to attribute not found");
                    // <body>
                    if let Ok(Event::Start(body_elem)) = reader.read_event() {
                        if body_elem.name().as_ref() != b"body" {
                            eyre::bail!("expected <body>");
                        }
                        // { text }
                        if let Ok(Event::Text(text_elem)) = reader.read_event() {
                            let body = String::from_utf8(text_elem.to_vec())
                                .expect("invalid utf8 body text");
                            // return parsed
                            Ok(Stanza::Message(StanzaMessage { to, body }))
                        } else {
                            eyre::bail!("failed to read body text")
                        }
                    } else {
                        eyre::bail!("failed to read body")
                    }
                },
                b"iq" => todo!(),
                b"presence" => todo!(),
                _ => eyre::bail!(format!(
                    "expected message/iq/presence got {:?}",
                    std::str::from_utf8(e.name().as_ref())
                )),
            }
        } else {
            eyre::bail!("failed to parse Stanza")
        }
    }
}

pub struct StanzaMessage {
    pub to: String,
    pub body: String,
}
