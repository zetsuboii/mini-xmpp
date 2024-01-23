//! `FromXml` and `ToXml` traits are used to convert between XML form and 
//! XMPP structs. `ToString` and `From<String>` methods are sometimes used
//! to represent XMPP structs as strings, so using them to serialize was not 
//! viable.

use color_eyre::eyre;

/// Trait to convert a struct into XML
pub trait ToXml<Xml = String> {
    fn to_xml(&self) -> Xml;
}

/// Trait to convert an XML type to struct
pub trait FromXml<Xml = String> {
    type Out;
    fn from_xml(xml: Xml) -> eyre::Result<Self::Out>;
}