use std::io::Cursor;

use base64::{prelude::BASE64_STANDARD as BASE64, Engine};
use color_eyre::eyre;
use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Reader, Writer,
};

use crate::Collect;

use super::serialize::{XmlCustomDeserialize, XmlCustomSerialize};

pub struct StreamHeader {
    pub from: String,
    pub to: String,
    pub version: String,
    pub xml_lang: String,
    pub xmlns: String,
    pub xmlns_stream: String,
}

impl XmlCustomSerialize for StreamHeader {
    fn into_string(&self) -> String {
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

impl XmlCustomDeserialize for StreamHeader {
    fn from_string(value: &str) -> eyre::Result<Self> {
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

impl XmlCustomSerialize for StreamHeaderResponse {
    fn into_string(&self) -> String {
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

impl XmlCustomDeserialize for StreamHeaderResponse {
    fn from_string(value: &str) -> eyre::Result<Self> {
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

impl XmlCustomSerialize for StreamFeatures {
    fn into_string(&self) -> String {
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

impl XmlCustomDeserialize for StreamFeatures {
    fn from_string(value: &str) -> eyre::Result<Self> {
        let mut reader = Reader::from_str(value);

        let mut header_found = false;
        let mut start_tls: Option<StartTls> = None;
        let mut mechanisms: Option<Mechanisms> = None;
        let mut bind: Option<Bind> = None;

        loop {
            if let Ok(event) = reader.read_event() {
                match event {
                    Event::Eof => break,
                    Event::Empty(e) => {
                        let name = e.name();
                        match name.as_ref() {
                            b"starttls" => {
                                if !header_found {
                                    eyre::bail!("header not found")
                                } else if start_tls.is_some() {
                                    eyre::bail!("starttls exists");
                                }

                                let xmlns = std::str::from_utf8(
                                    &e.try_get_attribute("xmlns").unwrap().unwrap().value,
                                )
                                .unwrap()
                                .to_string();

                                start_tls = Some(StartTls {
                                    xmlns,
                                    required: false,
                                });
                            }
                            b"bind" => {
                                if !header_found {
                                    eyre::bail!("header not found");
                                }

                                let xmlns = std::str::from_utf8(
                                    &e.try_get_attribute("xmlns").unwrap().unwrap().value,
                                )
                                .unwrap()
                                .to_string();

                                bind = Some(Bind { xmlns });
                            }
                            _ => {}
                        }
                    }
                    Event::Start(e) => {
                        let name = e.name();
                        match name.as_ref() {
                            b"stream:features" => header_found = true,
                            b"starttls" => {
                                if !header_found {
                                    eyre::bail!("header not found")
                                } else if start_tls.is_some() {
                                    eyre::bail!("starttls exists");
                                }

                                let xmlns = std::str::from_utf8(
                                    &e.try_get_attribute("xmlns").unwrap().unwrap().value,
                                )
                                .unwrap()
                                .to_string();

                                let mut required = false;

                                while let Ok(event) = reader.read_event() {
                                    match event {
                                        Event::Empty(e) => {
                                            if e.name().as_ref() == b"required" {
                                                required = true;
                                            }
                                        }
                                        Event::End(_) => {
                                            break;
                                        }
                                        _ => {}
                                    }
                                }

                                start_tls = Some(StartTls { xmlns, required });
                            }
                            b"mechanisms" => {
                                if !header_found {
                                    eyre::bail!("header not found")
                                }

                                let xmlns = std::str::from_utf8(
                                    &e.try_get_attribute("xmlns").unwrap().unwrap().value,
                                )
                                .unwrap()
                                .to_string();

                                let mut values = Vec::new();

                                while let Ok(event) = reader.read_event() {
                                    match event {
                                        Event::Text(text) => {
                                            let text =
                                                std::str::from_utf8(&text).unwrap().to_string();
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
                        }
                    }
                    _ => {}
                }
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

impl XmlCustomSerialize for StartTls {
    fn into_string(&self) -> String {
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

impl XmlCustomDeserialize for StartTls {
    fn from_string(value: &str) -> eyre::Result<Self> {
        let mut reader = Reader::from_str(value);

        while let Ok(event) = reader.read_event() {
            match event {
                Event::Empty(e) => {
                    let xmlns =
                        std::str::from_utf8(&e.try_get_attribute("xmlns").unwrap().unwrap().value)
                            .unwrap()
                            .to_string();

                    if e.name().as_ref() == b"starttls" {
                        return Ok(StartTls {
                            xmlns,
                            required: false,
                        });
                    }
                }
                Event::Start(e) => {
                    let xmlns =
                        std::str::from_utf8(&e.try_get_attribute("xmlns").unwrap().unwrap().value)
                            .unwrap()
                            .to_string();

                    let mut required = false;

                    while let Ok(event) = reader.read_event() {
                        match event {
                            Event::Empty(e) => {
                                if e.name().as_ref() == b"required" {
                                    required = true;
                                }
                            }
                            Event::End(_) => {
                                break;
                            }
                            _ => {}
                        }
                    }

                    return Ok(StartTls { xmlns, required });
                }
                _ => {}
            }
        }

        eyre::bail!("failed to parse")
    }
}

pub enum StartTlsResponse {
    Proceed(StartTlsProceed),
    Failure(StartTlsFailure),
}

impl XmlCustomDeserialize for StartTlsResponse {
    fn from_string(value: &str) -> eyre::Result<Self> {
        let mut reader = Reader::from_str(value);

        if let Ok(event) = reader.read_event() {
            match event {
                Event::Empty(e) => match e.name().as_ref() {
                    b"proceed" => return Ok(StartTlsResponse::Proceed(StartTlsProceed())),
                    b"failure" => return Ok(StartTlsResponse::Failure(StartTlsFailure())),
                    _ => {}
                },
                _ => {}
            }
        }
        eyre::bail!("invalid response")
    }
}

pub struct StartTlsProceed();

impl XmlCustomSerialize for StartTlsProceed {
    fn into_string(&self) -> String {
        let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));
        writer
            .write_event(Event::Empty(BytesStart::new("proceed")))
            .unwrap();
        writer.collect()
    }
}

pub struct StartTlsFailure();

impl XmlCustomSerialize for StartTlsFailure {
    fn into_string(&self) -> String {
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

impl XmlCustomDeserialize for Authentication {
    fn from_string(value: &str) -> eyre::Result<Self> {
        let mut reader = Reader::from_str(value);

        let mut xmlns: Option<String> = None;
        let mut mechanism: Option<Mechanism> = None;
        let mut value: Option<String> = None;

        while let Ok(event) = reader.read_event() {
            match event {
                Event::Start(e) => {
                    let name = e.name();
                    match name.as_ref() {
                        b"auth" => {
                            xmlns = Some(
                                std::str::from_utf8(
                                    &e.try_get_attribute("xmlns").unwrap().unwrap().value,
                                )
                                .unwrap()
                                .to_string(),
                            );
                            mechanism = Some(Mechanism(
                                std::str::from_utf8(
                                    &e.try_get_attribute("mechanism").unwrap().unwrap().value,
                                )
                                .unwrap()
                                .to_string(),
                            ));
                        }
                        _ => {}
                    }
                }
                Event::Text(text) => {
                    value = Some(std::str::from_utf8(&text).unwrap().to_string());
                }
                Event::End(_) => break,
                _ => {}
            }
        }

        Ok(Authentication {
            xmlns: xmlns.ok_or(eyre::eyre!("xmlns"))?,
            mechanism: mechanism.ok_or(eyre::eyre!("mechanism"))?,
            value: value.unwrap(),
        })
    }
}

impl XmlCustomSerialize for Authentication {
    fn into_string(&self) -> String {
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

    pub fn deserialize(value: String) -> Self {
        let value = BASE64.decode(value.as_bytes()).unwrap();
        let value = std::str::from_utf8(&value).unwrap();
        let value = value.split("\0").collect::<Vec<&str>>();
        Self::new(value[0].to_string(), value[1].to_string())
    }

    pub fn serialize(&self) -> String {
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

impl XmlCustomDeserialize for AuthenticationSuccess {
    fn from_string(value: &str) -> eyre::Result<Self> {
        let mut reader = Reader::from_str(value);
        let mut xmlns: Option<String> = None;

        while let Ok(event) = reader.read_event() {
            match event {
                Event::Empty(e) => {
                    xmlns = Some(
                        std::str::from_utf8(&e.try_get_attribute("xmlns").unwrap().unwrap().value)
                            .unwrap()
                            .to_string(),
                    );
                    break;
                }
                _ => {}
            }
        }

        Ok(AuthenticationSuccess {
            xmlns: xmlns.ok_or(eyre::eyre!("xmlns"))?,
        })
    }
}

impl XmlCustomSerialize for AuthenticationSuccess {
    fn into_string(&self) -> String {
        let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));
        // <success xmlns="urn:ietf:params:xml:ns:xmpp-sasl" />
        let mut success_start = BytesStart::new("success");
        success_start.push_attribute(("xmlns", self.xmlns.as_ref()));
        writer.write_event(Event::Empty(success_start)).unwrap();
        writer.collect()
    }
}
