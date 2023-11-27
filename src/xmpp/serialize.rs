use color_eyre::eyre;

pub trait XmlCustomSerialize {
    fn into_string(&self) -> String;
}

pub trait XmlCustomDeserialize where Self: Sized {
    fn from_string(value: &str) -> eyre::Result<Self>;
}