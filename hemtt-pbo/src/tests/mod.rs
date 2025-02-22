use std::{convert::TryInto, fs::File};

use crate::{
    Header, Timestamp, {ReadablePbo, WritablePbo},
};

pub fn pbo(
    file: File,
    file_count: usize,
    sorted: bool,
    extension_count: usize,
    version: &str,
    prefix: &str,
    checksum: Vec<u8>,
) -> ReadablePbo<File> {
    let mut pbo = ReadablePbo::from(file).unwrap();
    assert_eq!(pbo.files().len(), file_count);
    assert_eq!(pbo.extensions().len(), extension_count);
    assert_eq!(pbo.is_sorted().is_ok(), sorted);
    assert_eq!(pbo.extension("version"), Some(&version.to_string()));
    assert_eq!(pbo.extension("prefix"), Some(&prefix.to_string()));
    assert!(pbo.retrieve("not_real").is_none());
    assert!(pbo.header("not_real").is_none());
    if sorted {
        assert_eq!(pbo.checksum(), checksum);
    } else {
        assert_eq!(pbo.gen_checksum().unwrap(), checksum);
    }
    pbo
}

pub fn writeable_pbo(pbo: ReadablePbo<File>, file: File) {
    let mut writeable: WritablePbo<std::io::Cursor<Vec<u8>>> = pbo.try_into().unwrap();
    let original = ReadablePbo::from(file).unwrap();

    assert_eq!(original.files(), writeable.files_sorted().unwrap());
    assert_eq!(original.extensions(), writeable.extensions());
    assert_eq!(original.checksum(), writeable.checksum().unwrap());
}

pub fn header(
    header: &Header,
    filename: &str,
    method: u32,
    original: u32,
    reserved: u32,
    timestamp: Timestamp,
    size: u32,
) {
    assert_eq!(header.filename(), filename);
    assert_eq!(header.method(), method);
    assert_eq!(header.original(), original);
    assert_eq!(header.reserved(), reserved);
    assert_eq!(header.timestamp(), timestamp);
    assert_eq!(header.size(), size);
}

pub fn file(pbo: &mut ReadablePbo<File>, file: &str, content: String) {
    let data = pbo.retrieve(file).unwrap();
    let data = String::from_utf8(data.into_inner().to_vec()).unwrap();
    assert_eq!(data, content);
    assert_eq!(pbo.header(file).unwrap().size() as usize, data.len());
}
