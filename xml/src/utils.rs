use color_eyre::eyre;
use std::io::Cursor;

use quick_xml::{events::BytesStart, Writer};

/// Trait for converting a structure into string
pub trait Collect {
    /// Collect data as a `String` by consuming itself.
    fn collect(self) -> String;
}

impl Collect for Writer<Cursor<Vec<u8>>> {
    fn collect(self) -> String {
        String::from_utf8(self.into_inner().into_inner().as_slice().to_vec()).unwrap()
    }
}

/// Tries to get XML attribute from the starting header
///
/// ## Params
/// - `tag`: Starting tag
/// - `attribute`: Attribute as a string literal
pub fn try_get_attribute(tag: &BytesStart, attribute: &'static str) -> eyre::Result<String> {
    Ok(tag
        .try_get_attribute(attribute)?
        .ok_or(eyre::eyre!("xmlns not found"))
        .map(|attr| attr.value)
        .map(|value| String::from_utf8(value.into()))??)
}

pub const COLON_SEPARATOR: &'static str = "_COLON_";

#[inline]
pub fn escape_colon<T: AsRef<str>>(text: T) -> String {
    text.as_ref().replace(":", COLON_SEPARATOR)
}

#[inline]
pub fn unescape_colon<T: AsRef<str>>(text: T) -> String {
    text.as_ref().replace(COLON_SEPARATOR, ":")
}
