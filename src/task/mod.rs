mod handshake_task;
mod info_task;
mod peers_task;

use handshake_task::handshake_task;
use info_task::info_task;
use peers_task::peers_task;
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
        let _ = info_task(&args[2]);
    } else if command == "peers" {
        let _ = peers_task(&args[2]).await;
    } else if command == "handshake" {
        let _ = handshake_task(&args[2], &args[3]).await;
    } else {
        println!("unknown command: {}", args[1])
    };
    Ok(())
}
