use xmlserde_derives::{XmlSerialize, XmlDeserialize};

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