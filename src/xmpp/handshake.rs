use std::io::Cursor;

use base64::{prelude::BASE64_STANDARD as BASE64, Engine};
use color_eyre::eyre;
use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Reader, Writer,
};

use crate::{try_get_attribute, Collect};

pub struct StreamHeader {
    pub from: String,
    pub to: String,
    pub version: String,
    pub xml_lang: String,
    pub xmlns: String,
    pub xmlns_stream: String,
}

impl ToString for StreamHeader {
    fn to_string(&self) -> String {
        let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));

        let mut stream_header = BytesStart::new("stream:stream");
        stream_header.push_attribute(("from", self.from.as_str()));
        stream_header.push_attribute(("to", self.to.as_str()));
        stream_header.push_attribute(("version", self.version.as_str()));
        stream_header.push_attribute(("xml:lang", self.xml_lang.as_str()));
        stream_header.push_attribute(("xmlns", self.xmlns.as_str()));
        stream_header.push_attribute(("xmlns:stream", self.xmlns_stream.as_str()));

        writer.write_event(Event::Start(stream_header)).unwrap();
        writer.collect()
    }
}

impl TryFrom<&str> for StreamHeader {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut reader = Reader::from_str(value);

        let mut from: Option<String> = None;
        let mut to: Option<String> = None;
        let mut version: Option<String> = None;
        let mut xml_lang: Option<String> = None;
        let mut xmlns: Option<String> = None;
        let mut xmlns_stream: Option<String> = None;

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
                                let value = std::str::from_utf8(&attr.value).unwrap().to_string();

                                match key {
                                    b"from" => from = Some(value),
                                    b"to" => to = Some(value),
                                    b"version" => version = Some(value),
                                    b"xml:lang" => xml_lang = Some(value),
                                    b"xmlns" => xmlns = Some(value),
                                    b"xmlns:stream" => xmlns_stream = Some(value),
                                    _ => {}
                                }
                            }
                        })
                    }
                    _ => {}
                }
            }
        }

        return Ok(StreamHeader {
            from: from.ok_or(eyre::eyre!("from"))?,
            to: to.ok_or(eyre::eyre!("to"))?,
            version: version.ok_or(eyre::eyre!("version"))?,
            xml_lang: xml_lang.ok_or(eyre::eyre!("xml:lang"))?,
            xmlns: xmlns.ok_or(eyre::eyre!("xmlns"))?,
            xmlns_stream: xmlns_stream.ok_or(eyre::eyre!("xmlns:stream"))?,
        });
    }
}

impl StreamHeader {
    pub fn into_response(self, id: String) -> StreamHeaderResponse {
        StreamHeaderResponse {
            id,
            from: self.from,
            to: self.to,
            version: self.version,
            xml_lang: self.xml_lang,
            xmlns: self.xmlns,
            xmlns_stream: self.xmlns_stream,
        }
    }
}

pub struct StreamHeaderResponse {
    pub id: String,
    pub from: String,
    pub to: String,
    pub version: String,
    pub xml_lang: String,
    pub xmlns: String,
    pub xmlns_stream: String,
}

impl ToString for StreamHeaderResponse {
    fn to_string(&self) -> String {
        let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));

        let mut stream_header = BytesStart::new("stream:stream");
        stream_header.push_attribute(("id", self.id.as_str()));
        stream_header.push_attribute(("from", self.from.as_str()));
        stream_header.push_attribute(("to", self.to.as_str()));
        stream_header.push_attribute(("version", self.version.as_str()));
        stream_header.push_attribute(("xml:lang", self.xml_lang.as_str()));
        stream_header.push_attribute(("xmlns", self.xmlns.as_str()));
        stream_header.push_attribute(("xmlns:stream", self.xmlns_stream.as_str()));

        writer.write_event(Event::Start(stream_header)).unwrap();
        writer.collect()
    }
}

impl TryFrom<&str> for StreamHeaderResponse {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut reader = Reader::from_str(value);

        let mut id: Option<String> = None;
        let mut from: Option<String> = None;
        let mut to: Option<String> = None;
        let mut version: Option<String> = None;
        let mut xml_lang: Option<String> = None;
        let mut xmlns: Option<String> = None;
        let mut xmlns_stream: Option<String> = None;

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
                                let value = std::str::from_utf8(&attr.value).unwrap().to_string();

                                match key {
                                    b"id" => id = Some(value),
                                    b"from" => from = Some(value),
                                    b"to" => to = Some(value),
                                    b"version" => version = Some(value),
                                    b"xml:lang" => xml_lang = Some(value),
                                    b"xmlns" => xmlns = Some(value),
                                    b"xmlns:stream" => xmlns_stream = Some(value),
                                    _ => {}
                                }
                            }
                        })
                    }
                    _ => {}
                }
            }
        }

        return Ok(StreamHeaderResponse {
            id: id.ok_or(eyre::eyre!("id"))?,
            from: from.ok_or(eyre::eyre!("from"))?,
            to: to.ok_or(eyre::eyre!("to"))?,
            version: version.ok_or(eyre::eyre!("version"))?,
            xml_lang: xml_lang.ok_or(eyre::eyre!("xml:lang"))?,
            xmlns: xmlns.ok_or(eyre::eyre!("xmlns"))?,
            xmlns_stream: xmlns_stream.ok_or(eyre::eyre!("xmlns:stream"))?,
        });
    }
}

#[derive(Debug)]
pub struct StreamFeatures {
    pub start_tls: Option<StartTls>,
    pub mechanisms: Option<Mechanisms>,
    pub bind: Option<Bind>,
}

impl StreamFeatures {
    pub fn empty(&self) -> bool {
        self.start_tls.is_none() && self.mechanisms.is_none() && self.bind.is_none()
    }
}

impl ToString for StreamFeatures {
    fn to_string(&self) -> String {
        let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));
        let stream_features_start = BytesStart::new("stream:features");

        if self.empty() {
            // If there are no features, return <stream:features />
            writer
                .write_event(Event::Empty(stream_features_start))
                .unwrap();
            return writer.collect();
        }

        // <stream:features>
        writer
            .write_event(Event::Start(stream_features_start))
            .unwrap();

        if let Some(start_tls) = &self.start_tls {
            let mut start_tls_start = BytesStart::new("starttls");
            start_tls_start.push_attribute(("xmlns", start_tls.xmlns.as_ref()));

            if start_tls.required {
                // <starttls xmlns>
                writer.write_event(Event::Start(start_tls_start)).unwrap();
                // <required/>
                writer
                    .write_event(Event::Empty(BytesStart::new("required")))
                    .unwrap();
                let start_tls_end = BytesEnd::new("starttls");
                // </starttls>
                writer.write_event(Event::End(start_tls_end)).unwrap();
            } else {
                writer.write_event(Event::Empty(start_tls_start)).unwrap();
            }
        }

        if let Some(mechanisms) = &self.mechanisms {
            let mut mechanisms_start = BytesStart::new("mechanisms");
            mechanisms_start.push_attribute(("xmlns", mechanisms.xmlns.as_ref()));
            // <mechanisms>
            writer.write_event(Event::Start(mechanisms_start)).unwrap();

            for mechanism in &mechanisms.mechanisms {
                // <mechanism>
                writer
                    .write_event(Event::Start(BytesStart::new("mechanism")))
                    .unwrap();
                // Text
                writer
                    .write_event(Event::Text(BytesText::new(mechanism.0.as_ref())))
                    .unwrap();
                // </mechanism>
                writer
                    .write_event(Event::End(BytesEnd::new("mechanism")))
                    .unwrap();
            }

            // </mechanisms>
            writer
                .write_event(Event::End(BytesEnd::new("mechanisms")))
                .unwrap();
        }

        if let Some(bind) = &self.bind {
            // <bind xmlns={...} />
            let mut bind_start = BytesStart::new("bind");
            bind_start.push_attribute(("xmlns", bind.xmlns.as_ref()));
            writer.write_event(Event::Empty(bind_start)).unwrap();
        }

        // </stream:features>
        writer
            .write_event(Event::End(BytesEnd::new("stream:features")))
            .unwrap();
        writer.collect()
    }
}

impl TryFrom<&str> for StreamFeatures {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut reader = Reader::from_str(value);

        // <stream:features> or <stream:features/>
        match reader.read_event()? {
            Event::Start(_) => {}
            Event::Empty(_) => {
                return Ok(StreamFeatures {
                    bind: None,
                    start_tls: None,
                    mechanisms: None,
                })
            }
            _ => eyre::bail!("invalid xml"),
        }

        let mut start_tls: Option<StartTls> = None;
        let mut mechanisms: Option<Mechanisms> = None;
        let mut bind: Option<Bind> = None;

        while let Ok(event) = reader.read_event() {
            match event {
                // <starttls />
                // <bind />
                Event::Empty(tag) => match tag.name().as_ref() {
                    b"starttls" => {
                        if start_tls.is_some() {
                            eyre::bail!("starttls exists");
                        }
                        let xmlns = try_get_attribute(&tag, "xmlns")?;
                        start_tls = Some(StartTls {
                            xmlns,
                            required: false,
                        });
                    }
                    b"bind" => {
                        let xmlns = try_get_attribute(&tag, "xmlns")?;
                        bind = Some(Bind { xmlns });
                    }
                    _ => {}
                },
                // <starttls>
                // <mechanisms>
                Event::Start(tag) => match tag.name().as_ref() {
                    b"starttls" => {
                        if start_tls.is_some() {
                            eyre::bail!("starttls exists");
                        }

                        let xmlns = try_get_attribute(&tag, "xmlns")?;
                        let mut required = false;

                        // <required />
                        while let Ok(event) = reader.read_event() {
                            match event {
                                Event::Empty(e) => {
                                    if e.name().as_ref() == b"required" {
                                        required = true;
                                    }
                                }
                                Event::End(_) => break,
                                _ => {}
                            }
                        }

                        start_tls = Some(StartTls { xmlns, required });
                    }
                    b"mechanisms" => {
                        let xmlns = try_get_attribute(&tag, "xmlns")?;
                        let mut values = Vec::new();

                        while let Ok(event) = reader.read_event() {
                            match event {
                                Event::Text(text) => {
                                    let text = std::str::from_utf8(&text).unwrap().to_string();
                                    values.push(Mechanism(text));
                                }
                                Event::End(_) => break,
                                _ => {}
                            }
                        }

                        mechanisms = Some(Mechanisms {
                            xmlns,
                            mechanisms: values,
                        })
                    }
                    _ => {}
                },
                Event::End(tag) => {
                    if tag.name().as_ref() == b"stream:features" {
                        break;
                    }
                }
                _ => {}
            }
        }

        Ok(StreamFeatures {
            start_tls,
            mechanisms,
            bind,
        })
    }
}

#[derive(Debug)]
pub struct StartTls {
    pub xmlns: String,
    pub required: bool,
}

impl ToString for StartTls {
    fn to_string(&self) -> String {
        let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));
        let mut start_tls_start = BytesStart::new("starttls");
        start_tls_start.push_attribute(("xmlns", self.xmlns.as_ref()));

        if self.required {
            // <starttls xmlns>
            writer.write_event(Event::Start(start_tls_start)).unwrap();
            // <required/>
            writer
                .write_event(Event::Empty(BytesStart::new("required")))
                .unwrap();
            let start_tls_end = BytesEnd::new("starttls");
            // </starttls>
            writer.write_event(Event::End(start_tls_end)).unwrap();
        } else {
            writer.write_event(Event::Empty(start_tls_start)).unwrap();
        }

        writer.collect()
    }
}

impl TryFrom<&str> for StartTls {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut reader = Reader::from_str(value);

        let start_tls_start = reader.read_event()?;
        let (start_tls_start, empty) = match start_tls_start {
            Event::Empty(tag) => (tag, true),
            Event::Start(tag) => (tag, false),
            _ => eyre::bail!("invalid xml"),
        };

        let xmlns = try_get_attribute(&start_tls_start, "xmlns")?;

        if start_tls_start.name().as_ref() != b"starttls" {
            eyre::bail!("invalid tag name")
        }

        if empty {
            return Ok(StartTls {
                xmlns,
                required: false,
            });
        }

        let required = match reader.read_event()? {
            Event::Empty(tag) => tag.name().as_ref() == b"required",
            _ => false,
        };

        Ok(StartTls { xmlns, required })
    }
}

pub enum StartTlsResponse {
    Proceed(StartTlsProceed),
    Failure(StartTlsFailure),
}

impl TryFrom<&str> for StartTlsResponse {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut reader = Reader::from_str(value);

        let tag = match reader.read_event()? {
            Event::Empty(tag) => tag,
            _ => eyre::bail!("invalid xml"),
        };

        let result = match tag.name().as_ref() {
            b"proceed" => StartTlsResponse::Proceed(StartTlsProceed()),
            b"failure" => StartTlsResponse::Failure(StartTlsFailure()),
            _ => eyre::bail!("invalid result"),
        };

        Ok(result)
    }
}

pub struct StartTlsProceed();

impl ToString for StartTlsProceed {
    fn to_string(&self) -> String {
        let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));
        writer
            .write_event(Event::Empty(BytesStart::new("proceed")))
            .unwrap();
        writer.collect()
    }
}

pub struct StartTlsFailure();

impl ToString for StartTlsFailure {
    fn to_string(&self) -> String {
        let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));
        writer
            .write_event(Event::Empty(BytesStart::new("failure")))
            .unwrap();
        writer.collect()
    }
}

#[derive(Debug)]
pub struct Mechanisms {
    pub xmlns: String,
    pub mechanisms: Vec<Mechanism>,
}

#[derive(Debug)]
pub struct Mechanism(pub String);

#[derive(Debug)]
pub struct Bind {
    pub xmlns: String,
}

pub struct Authentication {
    pub xmlns: String,
    pub mechanism: Mechanism,
    pub value: String,
}

impl Authentication {
    pub fn new(xmlns: String, mechanism: Mechanism, value: String) -> Self {
        Self {
            xmlns,
            mechanism,
            value,
        }
    }

    pub fn deserialize_credentials(&self) -> Credentials {
        let value = BASE64.decode(self.value.as_bytes()).unwrap();
        let value = std::str::from_utf8(&value).unwrap();
        let value = value.split("\0").collect::<Vec<&str>>();
        Credentials::new(value[1].to_string(), value[2].to_string())
    }
}

impl TryFrom<&str> for Authentication {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut reader = Reader::from_str(value);

        // <auth xmlns="urn:ietf:params:xml:ns:xmpp-sasl" mechanism="PLAIN">
        let auth_start = match reader.read_event()? {
            Event::Start(tag) => tag,
            _ => eyre::bail!("invalid xml"),
        };
        if auth_start.name().as_ref() != b"auth" {
            eyre::bail!("invalid tag name")
        }
        let xmlns = try_get_attribute(&auth_start, "xmlns")?;
        let mechanism = try_get_attribute(&auth_start, "mechanism").map(|name| Mechanism(name))?;

        // {...}
        let text_tag = match reader.read_event()? {
            Event::Text(text) => text,
            _ => eyre::bail!("invalid xml"),
        };
        let value = String::from_utf8(text_tag.as_ref().into())?;

        Ok(Authentication {
            xmlns,
            mechanism,
            value,
        })
    }
}

impl ToString for Authentication {
    fn to_string(&self) -> String {
        let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));
        // <auth xmlns="urn:ietf:params:xml:ns:xmpp-sasl" mechanism="PLAIN">
        let mut auth_start = BytesStart::new("auth");
        auth_start.push_attribute(("xmlns", self.xmlns.as_ref()));
        auth_start.push_attribute(("mechanism", self.mechanism.0.as_ref()));

        // {...}
        writer.write_event(Event::Start(auth_start)).unwrap();
        writer
            .write_event(Event::Text(BytesText::new(self.value.as_ref())))
            .unwrap();

        // </auth>
        writer
            .write_event(Event::End(BytesEnd::new("auth")))
            .unwrap();
        writer.collect()
    }
}

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub fn from_base64(value: String) -> Self {
        let value = BASE64.decode(value.as_bytes()).unwrap();
        let value = std::str::from_utf8(&value).unwrap();
        let value = value.split("\0").collect::<Vec<&str>>();
        Self::new(value[0].to_string(), value[1].to_string())
    }

    pub fn to_base64(&self) -> String {
        let mut serialized = String::new();
        serialized.push_str(&self.username.as_str());
        serialized.push('\0');
        serialized.push_str(&self.password.as_str());
        BASE64.encode(serialized)
    }
}

pub struct AuthenticationSuccess {
    pub xmlns: String,
}

impl AuthenticationSuccess {
    pub fn new(xmlns: String) -> Self {
        Self { xmlns }
    }
}

impl TryFrom<&str> for AuthenticationSuccess {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut reader = Reader::from_str(value);

        let tag = match reader.read_event()? {
            Event::Empty(tag) => tag,
            _ => eyre::bail!("invalid xml"),
        };

        if tag.name().as_ref() != b"success" {
            eyre::bail!("invalid tag name");
        }

        let xmlns = try_get_attribute(&tag, "xmlns")?;

        Ok(AuthenticationSuccess { xmlns })
    }
}

impl ToString for AuthenticationSuccess {
    fn to_string(&self) -> String {
        let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));
        // <success xmlns="urn:ietf:params:xml:ns:xmpp-sasl" />
        let mut success_start = BytesStart::new("success");
        success_start.push_attribute(("xmlns", self.xmlns.as_ref()));
        writer.write_event(Event::Empty(success_start)).unwrap();
        writer.collect()
    }
}
