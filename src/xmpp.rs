use xmlserde_derives::{XmlDeserialize, XmlSerialize};

/// NOTE: This is not a valid stream header, will migrate to minidom or an
/// equivalent
#[allow(unused)]
#[derive(XmlSerialize, XmlDeserialize, Clone)]
#[xmlserde(root = b"stream:stream")]
pub struct InitialStreamHeader {
    #[xmlserde(name = b"from", ty = "attr")]
    pub from: String,
    #[xmlserde(name = b"to", ty = "attr")]
    pub to: String,
    #[xmlserde(name = b"version", ty = "attr")]
    pub version: String,
    #[xmlserde(name = b"xml:lang", ty = "attr")]
    pub xml_lang: String,
    #[xmlserde(name = b"xmlns", ty = "attr")]
    pub xmlns: String,
    #[xmlserde(name = b"xmlns:stream", ty = "attr")]
    pub xmlns_stream: String,
}

impl InitialStreamHeader {
    pub fn into_response(self, id: String) -> ResponseStreamHeader {
        ResponseStreamHeader {
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

/// NOTE: This is not a valid stream header, will migrate to minidom or an
/// equivalent
#[allow(unused)]
#[derive(XmlSerialize, XmlDeserialize)]
#[xmlserde(root = b"stream:stream")]
pub struct ResponseStreamHeader {
    #[xmlserde(name = b"id", ty = "attr")]
    pub id: String,
    #[xmlserde(name = b"from", ty = "attr")]
    pub from: String,
    #[xmlserde(name = b"to", ty = "attr")]
    pub to: String,
    #[xmlserde(name = b"version", ty = "attr")]
    pub version: String,
    #[xmlserde(name = b"xml:lang", ty = "attr")]
    pub xml_lang: String,
    #[xmlserde(name = b"xmlns", ty = "attr")]
    pub xmlns: String,
    #[xmlserde(name = b"xmlns:stream", ty = "attr")]
    pub xmlns_stream: String,
}

#[derive(XmlSerialize, XmlDeserialize)]
#[xmlserde(root = b"stream:features")]
pub struct StreamFeatures {
    #[xmlserde(name=b"starttls", ty="child")]
    pub start_tls: Option<StartTls>,
    #[xmlserde(name=b"mechanisms", ty="child")]
    pub mechanisms: Option<Mechanisms>
}

#[derive(XmlSerialize, XmlDeserialize)]
#[xmlserde(root = b"starttls")]
pub struct StartTls {
    #[xmlserde(name=b"xmlns", ty = "attr")]
    pub xmlns: String,
    #[xmlserde(name=b"required", ty="child")]
    pub required: Option<StartTlsRequired>
}

#[derive(XmlSerialize, XmlDeserialize)]
pub struct StartTlsRequired();

#[derive(XmlSerialize, XmlDeserialize)]
#[xmlserde(root = b"proceed")]
pub struct StartTlsProceed();

#[derive(XmlSerialize, XmlDeserialize)]
#[xmlserde(root = b"failure")]
pub struct StartTlsFailure();

#[derive(XmlSerialize, XmlDeserialize)]
pub struct Mechanisms {
    #[xmlserde(name=b"xmlns", ty = "attr")]
    pub xmlns: String,
    #[xmlserde(name=b"mechanism", ty="child")]
    pub mechanisms: Vec<Mechanism>
}

#[derive(XmlSerialize, XmlDeserialize)]
pub struct Mechanism {
    #[xmlserde(ty="text")]
    pub value: String
}