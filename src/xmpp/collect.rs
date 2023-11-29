use std::io::Cursor;

use quick_xml::Writer;

pub trait Collect {
    fn collect(self) -> String;
}

impl Collect for Writer<Cursor<Vec<u8>>> {
    fn collect(self) -> String {
        String::from_utf8(self.into_inner().into_inner().as_slice().to_vec()).unwrap()
    }
}
