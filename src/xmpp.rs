use xmlserde_derives::{XmlDeserialize, XmlSerialize};

#[allow(unused)]
#[derive(XmlSerialize, XmlDeserialize)]
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
