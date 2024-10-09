mod info_task;
mod peers_task;

use info_task::info;
use peers_task::peers;
use serde_bencode::value::Value;
use std::env;

use crate::{display_value, MyBEncodedBuf};

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &mut MyBEncodedBuf) -> Value {
    let a = encoded_value.decode().unwrap();
    a
}
pub async fn torrent_task() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];
    if command == "decode" {
        let encoded_value = &args[2];
        let mut buf = MyBEncodedBuf::from(encoded_value);
        let decoded_value = decode_bencoded_value(&mut buf);
        let _ = buf.encode(&decoded_value);
        display_value(&decoded_value);
    } else if command == "info" {
        let _ = info(&args[2]);
    } else if command == "peers" {
        let _ = peers(&args[2]).await;
    } else {
        println!("unknown command: {}", args[1])
    };
    Ok(())
}
