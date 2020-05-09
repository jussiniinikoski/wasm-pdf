use std::io::Write;
extern crate deflate;

use deflate::write::ZlibEncoder;
use deflate::Compression;

pub fn encode(input: &[u8]) -> Result<Vec<u8>, String> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(input).unwrap();
    let compressed_bytes = e.finish();
    let bytes = compressed_bytes.unwrap();
    Ok(bytes)
}
