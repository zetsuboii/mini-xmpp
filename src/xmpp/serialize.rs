use color_eyre::eyre;
use quick_xml::events::BytesStart;

pub trait XmlCustomSerialize {
    fn into_string(&self) -> String;
}

pub trait XmlCustomDeserialize
where
    Self: Sized,
{
    fn from_string(value: &str) -> eyre::Result<Self>;
}

pub fn try_get_attribute(tag: &BytesStart, attribute: &'static str) -> eyre::Result<String> {
    Ok(tag
        .try_get_attribute(attribute)?
        .ok_or(eyre::eyre!("xmlns not found"))
        .map(|attr| attr.value)
        .map(|value| String::from_utf8(value.into()))??)
}
