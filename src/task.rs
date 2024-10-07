use serde_bencode::value::Value;
use sha1::{Digest, Sha1};
use std::env;

use crate::{
    dict_get, dict_get_as, display_value, value_as_bytes, value_as_int, MyBEncodedBuf,
    MyTorrentResult,
};

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &mut MyBEncodedBuf) -> Value {
    let a = encoded_value.decode().unwrap();
    a
}

// Usage: 70edcac2611a8829ebf467a6849f5d8408d9d8f4
pub fn torrent_task() -> MyTorrentResult<()> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];
    if command == "decode" {
        let encoded_value = &args[2];
        let mut buf = MyBEncodedBuf::from(encoded_value);
        let decoded_value = decode_bencoded_value(&mut buf);
        let _ = buf.encode(&decoded_value);
        display_value(&decoded_value);
    } else if command == "info" {
        let file_name = &args[2];
        let a = std::fs::read(file_name)?;
        let mut buf = MyBEncodedBuf {
            pos: 0,
            inner_buf: a,
            outer_buf: vec![],
        };
        let decoded_value = decode_bencoded_value(&mut buf);
        display_value(&decoded_value);

        let info_value = dict_get(&decoded_value, "info")?;

        let announce_vec = dict_get_as(&decoded_value, "announce", |v| value_as_bytes(v))?;

        let length = dict_get_as(&info_value, "length", |v| value_as_int(v))?;
        let piece_length = dict_get_as(&info_value, "piece length", |v| value_as_int(v))?;
        let pieces = dict_get_as(&info_value, "pieces", |v| value_as_bytes(v))?;

        let _ = buf.encode(&info_value);

        let _sh1_digest = Sha1::digest(&buf.outer_buf);
        let hx = hex::encode(_sh1_digest);

        println!("Tracker URL: {}", String::from_utf8(announce_vec)?);
        println!("Length: {}", length);
        println!("Info Hash: {} ", hx);
        println!("Piece Length: {}", piece_length);
        println!("Piece Hashes: \n{}", pieces_hash(&info_value)?.join("\n"));
    } else {
        println!("unknown command: {}", args[1])
    };
    Ok(())
}
pub fn pieces_hash(v: &Value) -> MyTorrentResult<Vec<String>> {
    let pieces = dict_get_as(v, "pieces", |v| value_as_bytes(v))?;
    let a: Vec<_> = pieces
        .chunks(20)
        .map(|c| {
            let s: String = hex::encode(c);
            return s;
        })
        .collect();

    Ok(a)
}
