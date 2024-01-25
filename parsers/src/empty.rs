/// Trait to indicate if XML element is empty or not
/// 
/// By default, it returns false, meaning while XML is deserialized
/// it will default to <tag> </tag> instead of <tag />
pub trait IsEmpty {
    fn is_empty(&self) -> bool {
        false
    }
}