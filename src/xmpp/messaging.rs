use std::io::Cursor;

use color_eyre::eyre;
use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Reader, Writer,
};

use crate::{try_get_attribute, Collect, Jid};

#[derive(Debug)]
pub enum Stanza {
    Message(StanzaMessage),
    Presence(StanzaPresence),
    Iq(StanzaIq),
}

impl ToString for Stanza {
    fn to_string(&self) -> String {
        let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));

        match self {
            Self::Message(message) => {
                // <message from={...} to={...}>
                let mut message_start = BytesStart::new("message");
                if let Some(id) = &message.id {
                    message_start.push_attribute(("id", id.as_ref()));
                }
                if let Some(from) = &message.from {
                    message_start.push_attribute(("from", from.as_ref()));
                }
                if let Some(to) = &message.to {
                    message_start.push_attribute(("to", to.as_ref()));
                }
                if let Some(xml_lang) = &message.xml_lang {
                    message_start.push_attribute(("xml:lang", xml_lang.as_ref()));
                }

                writer.write_event(Event::Start(message_start)).unwrap();

                if let Some(body) = &message.body {
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
                writer
                    .write_event(Event::End(BytesEnd::new("message")))
                    .unwrap();
            }
            Self::Iq(iq) => {
                let StanzaIq {
                    id,
                    from,
                    type_,
                    payload,
                } = iq;

                // <iq id={...} type={...}>
                let mut iq_header = BytesStart::new("iq");
                if let Some(id) = id {
                    iq_header.push_attribute(("id", id.as_ref()));
                }
                if let Some(from) = from {
                    iq_header.push_attribute(("from", from.as_ref()));
                }
                if let Some(type_) = type_ {
                    iq_header.push_attribute(("type", type_.as_ref()));
                }
                writer.write_event(Event::Start(iq_header)).unwrap();

                match payload {
                    StanzaIqPayload::Bind(payload) => {
                        // <bind xmlns={...} >
                        let mut bind_start = BytesStart::new("bind");
                        bind_start.push_attribute(("xmlns", payload.xmlns.as_ref()));

                        if payload.resource.is_none() && payload.jid.is_none() {
                            writer.write_event(Event::Empty(bind_start)).unwrap();
                        } else {
                            writer.write_event(Event::Start(bind_start)).unwrap();

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
                    StanzaIqPayload::Friends(payload) => {
                        let IqFriendsPayload { xmlns, friend_list } = payload;
                        let mut friends_start = BytesStart::new("friends");
                        friends_start.push_attribute(("xmlns", xmlns.as_ref()));

                        if let Some(friend_list) = friend_list {
                            // <friends>
                            writer.write_event(Event::Start(friends_start)).unwrap();

                            for friend in friend_list {
                                // <jid>
                                writer
                                    .write_event(Event::Start(BytesStart::new("jid")))
                                    .unwrap();
                                // {...}
                                writer
                                    .write_event(Event::Text(BytesText::new(
                                        friend.to_string().as_str(),
                                    )))
                                    .unwrap();
                                // </jid>
                                writer
                                    .write_event(Event::End(BytesEnd::new("jid")))
                                    .unwrap();
                            }

                            // </friends>
                            let friends_end = BytesEnd::new("friends");
                            writer.write_event(Event::End(friends_end)).unwrap();
                        } else {
                            // <friends />
                            writer.write_event(Event::Empty(friends_start)).unwrap();
                        }
                    }
                }

                // </iq>
                let iq_end = BytesEnd::new("iq");
                writer.write_event(Event::End(iq_end)).unwrap();
            }
            Self::Presence(presence) => {
                let StanzaPresence { id, from } = presence;

                // <presence id={...} from={...}>
                let mut presence_header = BytesStart::new("presence");
                if let Some(id) = id {
                    presence_header.push_attribute(("id", id.as_ref()));
                }
                if let Some(from) = from {
                    presence_header.push_attribute(("from", from.as_ref()));
                }
                writer.write_event(Event::Start(presence_header)).unwrap();

                // </presence>
                let presence_end = BytesEnd::new("presence");
                writer.write_event(Event::End(presence_end)).unwrap();
            }
        }

        writer.collect()
    }
}

impl TryFrom<&str> for Stanza {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut reader = Reader::from_str(value);
        let start_tag = match reader.read_event()? {
            Event::Start(tag) => tag,
            Event::Empty(tag) => tag,
            _ => eyre::bail!("invalid xml"),
        };

        match start_tag.name().as_ref() {
            b"message" => {
                let id = try_get_attribute(&start_tag, "id").ok();
                let from = try_get_attribute(&start_tag, "from").ok();
                let to = try_get_attribute(&start_tag, "to").ok();
                let xml_lang = try_get_attribute(&start_tag, "xml:lang").ok();

                // <body>
                if let Ok(Event::Start(body_elem)) = reader.read_event() {
                    if body_elem.name().as_ref() != b"body" {
                        eyre::bail!("expected <body>");
                    }
                    // { text }
                    if let Ok(Event::Text(body_text)) = reader.read_event() {
                        let body = String::from_utf8(body_text.as_ref().into()).ok();
                        // return parsed
                        Ok(Stanza::Message(StanzaMessage {
                            id,
                            from,
                            to,
                            body,
                            xml_lang,
                        }))
                    } else {
                        eyre::bail!("failed to read body text")
                    }
                } else {
                    eyre::bail!("failed to read body")
                }
            }
            b"iq" => {
                // attribute `id`
                let id = try_get_attribute(&start_tag, "id").ok();
                // attribute `from`
                let from = try_get_attribute(&start_tag, "from").ok();
                // attribute `type`
                let type_ = try_get_attribute(&start_tag, "type").expect("type");

                let mut iq_payload: Option<StanzaIqPayload> = None;

                while let Ok(payload_event) = reader.read_event() {
                    match payload_event {
                        Event::Empty(tag) => match tag.name().as_ref() {
                            // <bind />
                            b"bind" => {
                                let xmlns = tag
                                    .try_get_attribute("xmlns")
                                    .map(|attr| attr.ok_or(eyre::eyre!("attr not found")))?
                                    .map(|attr| attr.value)
                                    .map(|value| String::from_utf8(value.into()))??;

                                iq_payload = Some(StanzaIqPayload::Bind(IqBindPayload {
                                    xmlns,
                                    jid: None,
                                    resource: None,
                                }));
                            }
                            b"friends" => {
                                let xmlns = tag
                                    .try_get_attribute("xmlns")
                                    .map(|attr| attr.ok_or(eyre::eyre!("attr not found")))?
                                    .map(|attr| attr.value)
                                    .map(|value| String::from_utf8(value.into()))??;

                                iq_payload = Some(StanzaIqPayload::Friends(IqFriendsPayload {
                                    xmlns,
                                    friend_list: None,
                                }));
                            }
                            _ => {}
                        },
                        Event::Start(tag) => match tag.name().as_ref() {
                            b"bind" => {
                                let xmlns = tag
                                    .try_get_attribute("xmlns")?
                                    .ok_or(eyre::eyre!("xmlns not found"))
                                    .map(|attr| attr.value)
                                    .map(|value| String::from_utf8(value.into()))??;

                                let mut bind_payload = IqBindPayload {
                                    xmlns,
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
                                                        String::from_utf8(text.to_vec()).unwrap(),
                                                    );
                                                }
                                            } else if tag.name().as_ref() == b"resource" {
                                                let text_event = reader.read_event();
                                                if let Ok(Event::Text(text)) = text_event {
                                                    bind_payload.resource = Some(
                                                        String::from_utf8(text.to_vec()).unwrap(),
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
                            b"friends" => {
                                let xmlns = tag
                                    .try_get_attribute("xmlns")?
                                    .ok_or(eyre::eyre!("xmlns not found"))
                                    .map(|attr| attr.value)
                                    .map(|value| String::from_utf8(value.into()))??;

                                let mut friends_payload = IqFriendsPayload {
                                    xmlns,
                                    friend_list: None,
                                };

                                let mut friend_list = Vec::new();
                                while let Ok(bind_event) = reader.read_event() {
                                    match bind_event {
                                        Event::Start(tag) => {
                                            if tag.name().as_ref() == b"jid" {
                                                let text_event = reader.read_event();
                                                if let Ok(Event::Text(text)) = text_event {
                                                    friend_list.push(
                                                        Jid::try_from(
                                                            std::str::from_utf8(text.as_ref())
                                                                .unwrap(),
                                                        )
                                                        .unwrap(),
                                                    )
                                                }
                                            }
                                        }
                                        Event::End(tag) => {
                                            if tag.name().as_ref() == b"friends" {
                                                break;
                                            }
                                        }
                                        _ => {}
                                    }
                                }

                                if friend_list.len() > 0 {
                                    friends_payload.friend_list = Some(friend_list)
                                }
                                iq_payload = Some(StanzaIqPayload::Friends(friends_payload));
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
                    id,
                    from,
                    type_: Some(type_),
                    payload: iq_payload.expect("found empty payload"),
                }))
            }
            b"presence" => {
                let id = try_get_attribute(&start_tag, "id").ok();
                let from = try_get_attribute(&start_tag, "from").ok();
                Ok(Stanza::Presence(StanzaPresence { id, from }))
            }
            _ => eyre::bail!("invalid stanza"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StanzaMessage {
    pub id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub body: Option<String>,
    pub xml_lang: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StanzaPresence {
    pub id: Option<String>,
    pub from: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StanzaIq {
    pub id: Option<String>,
    pub from: Option<String>,
    pub type_: Option<String>,
    pub payload: StanzaIqPayload,
}

#[derive(Debug, Clone)]
pub enum StanzaIqPayload {
    Bind(IqBindPayload),
    Friends(IqFriendsPayload),
}

#[derive(Debug, Clone)]
pub struct IqBindPayload {
    pub xmlns: String,
    pub jid: Option<String>,
    pub resource: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IqFriendsPayload {
    pub xmlns: String,
    pub friend_list: Option<Vec<Jid>>,
}
