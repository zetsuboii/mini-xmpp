use std::io::Cursor;

use color_eyre::eyre;
use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Reader, Writer,
};

use crate::{Collect, XmlCustomDeserialize, XmlCustomSerialize};

fn get_attr_or_panic(tag: &BytesStart, attribute: &'static str) -> String {
    String::from_utf8(
        tag.try_get_attribute(attribute)
            .unwrap()
            .unwrap()
            .value
            .to_vec(),
    )
    .expect(&format!("attribute {attribute} not found"))
}

pub enum Stanza {
    Message(StanzaMessage),
    Presence,
    Iq(StanzaIq),
}

impl XmlCustomSerialize for Stanza {
    fn into_string(&self) -> String {
        let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));

        match self {
            Self::Message(message) => {
                // <message to={...}>
                let mut message_header = BytesStart::new("message");
                message_header.push_attribute(("to", message.to.as_ref()));
                writer.write_event(Event::Start(message_header)).unwrap();
                // <body>
                writer
                    .write_event(Event::Start(BytesStart::new("body")))
                    .unwrap();
                // {...}
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
            Self::Iq(iq) => {
                let StanzaIq {
                    iq_id,
                    iq_type,
                    iq_payload: iq_inner,
                } = iq;

                // <iq id={...} type={...}>
                let mut iq_header = BytesStart::new("iq");
                iq_header.push_attribute(("id", iq_id.as_ref()));
                iq_header.push_attribute(("type", iq_type.as_ref()));
                writer.write_event(Event::Start(iq_header)).unwrap();

                match iq_inner {
                    StanzaIqPayload::Bind(payload) => {
                        // <bind>
                        writer
                            .write_event(Event::Start(BytesStart::new("bind")))
                            .unwrap();

                        if let Some(resource) = &payload.resource {
                            // <resource>
                            writer
                                .write_event(Event::Start(BytesStart::new("resource")))
                                .unwrap();
                            // {...}
                            writer
                                .write_event(Event::Text(BytesText::new(resource.as_ref())))
                                .unwrap();
                            // </resource>
                            writer
                                .write_event(Event::End(BytesEnd::new("resource")))
                                .unwrap();
                        }

                        if let Some(jid) = &payload.jid {
                            // <jid>
                            writer
                                .write_event(Event::Start(BytesStart::new("jid")))
                                .unwrap();
                            // {...}
                            writer
                                .write_event(Event::Text(BytesText::new(jid.as_ref())))
                                .unwrap();
                            // </jid>
                            writer
                                .write_event(Event::End(BytesEnd::new("jid")))
                                .unwrap();
                        }
                        // </bind>
                        writer
                            .write_event(Event::End(BytesEnd::new("bind")))
                            .unwrap();
                    }
                }
                // </iq>
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
                    let to = get_attr_or_panic(&e, "to");

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
                }
                b"iq" => {
                    // attribute `id`
                    let iq_id = get_attr_or_panic(&e, "id");
                    // attribute `type`
                    let iq_type = get_attr_or_panic(&e, "type");

                    let mut iq_payload: Option<StanzaIqPayload> = None;

                    while let Ok(payload_event) = reader.read_event() {
                        match payload_event {
                            Event::Start(tag) => match tag.name().as_ref() {
                                b"bind" => {
                                    let mut bind_payload = IqBindPayload {
                                        jid: None,
                                        resource: None,
                                    };
                                    while let Ok(bind_event) = reader.read_event() {
                                        match bind_event {
                                            Event::Start(tag) => {
                                                if tag.name().as_ref() == b"jid" {
                                                    let text_event = reader.read_event();
                                                    if let Ok(Event::Text(text)) = text_event {
                                                        bind_payload.jid = Some(
                                                            String::from_utf8(text.to_vec())
                                                                .unwrap(),
                                                        );
                                                    }
                                                } else if tag.name().as_ref() == b"resource" {
                                                    let text_event = reader.read_event();
                                                    if let Ok(Event::Text(text)) = text_event {
                                                        bind_payload.resource = Some(
                                                            String::from_utf8(text.to_vec())
                                                                .unwrap(),
                                                        );
                                                    }
                                                }
                                            }
                                            Event::End(tag) => {
                                                if tag.name().as_ref() == b"bind" {
                                                    break;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    iq_payload = Some(StanzaIqPayload::Bind(bind_payload));
                                }
                                _ => {}
                            },
                            Event::End(tag) => {
                                if tag.name().as_ref() == b"iq" {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }

                    Ok(Stanza::Iq(StanzaIq {
                        iq_type,
                        iq_id,
                        iq_payload: iq_payload.expect("found empty payload"),
                    }))
                }
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

pub struct StanzaIq {
    pub iq_type: String,
    pub iq_id: String,
    pub iq_payload: StanzaIqPayload,
}

pub enum StanzaIqPayload {
    Bind(IqBindPayload),
}

pub struct IqBindPayload {
    pub jid: Option<String>,
    pub resource: Option<String>,
}
