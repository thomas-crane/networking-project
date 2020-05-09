use std::convert::TryInto;

pub fn create_payload(size: usize) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::with_capacity(size);
    let max = std::u8::MAX as usize;
    for i in 0..size {
        buf.push((i % max).try_into().expect("overflow while creating data"));
    }
    buf
}
