use std::env;

use crate::{display_value, MyBEncodedBuf};

pub async fn decode_task(encoded_value: &str) -> anyhow::Result<()> {
    let mut buf = MyBEncodedBuf::from(encoded_value);
    let decoded_value = buf.decode().unwrap();
    let _ = buf.encode(&decoded_value);
    display_value(&decoded_value);
    Ok(())
}
