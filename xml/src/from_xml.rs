//! `FromXml` and `ToXml` traits are used to convert between XML form and
//! XMPP structs. `ToString` and `From<String>` methods are sometimes used
//! to represent XMPP structs as strings, so using them to serialize was not
//! viable.

use std::io::Cursor;

use color_eyre::eyre;
use quick_xml::{events::Event, Reader, Writer};

use crate::utils::Collect;

pub trait ReadXml<'r, R = &'r [u8], Out = Self> {
    /// Reads XML starting from the root event
    fn read_xml(root: Event, reader: &mut Reader<R>) -> eyre::Result<Out>;
}

/// Trait to read XML from a string
pub trait ReadXmlString<'r>: ReadXml<'r>
where
    Self: Sized,
{
    /// Reads XML from a string and returns `Result<Self>`
    fn read_xml_string(xml: &'r str) -> eyre::Result<Self> {
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let root = reader.read_event()?;
        Self::read_xml(root, &mut reader)
    }
}

/// Blanket implementation for `ReadXmlString` for all `ReadXml` types
impl<'a, T: ReadXml<'a>> ReadXmlString<'a> for T {}

pub trait WriteXml<W = Cursor<Vec<u8>>, Out = ()> {
    /// Writes XML to the writer
    fn write_xml(&self, writer: &mut Writer<W>) -> eyre::Result<Out>;
}

pub trait WriteXmlString: WriteXml {
    /// Writes XML to a string
    fn write_xml_string(&self) -> eyre::Result<String> {
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        self.write_xml(&mut writer)?;
        Ok(writer.collect())
    }
}

/// Blanket implementation for `WriteXmlString` for all `WriteXml` types
impl<T: WriteXml> WriteXmlString for T {}
