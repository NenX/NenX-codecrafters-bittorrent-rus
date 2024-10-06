use bittorrent_starter_rust::MyBEncodedBuf;
use serde_json::{self};
use sha1::{Digest, Sha1};
use std::{env, error::Error};

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &mut MyBEncodedBuf) -> serde_json::Value {
    let a = encoded_value.decode().unwrap();
    a
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];
    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        // println!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let mut buf = MyBEncodedBuf::from(encoded_value);
        let decoded_value = decode_bencoded_value(&mut buf);
        let _ = buf.encode(&decoded_value);
        println!("{} {}", encoded_value, decoded_value.to_string());
    } else if command == "info" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        // println!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let file_name = &args[2];
        let a = std::fs::read(file_name).unwrap();
        let mut buf = MyBEncodedBuf {
            pos: 0,
            inner_buf: a,
            outer_buf: vec![],
        };
        let decoded_value = decode_bencoded_value(&mut buf);
        let announce = decoded_value
            .as_object()
            .unwrap()
            .get("announce")
            .unwrap()
            .as_str()
            .unwrap();
        let info_v = decoded_value.as_object().unwrap().get("info").unwrap();
        let info = info_v.as_object().unwrap();
        let length = info.get("length").unwrap().as_i64().unwrap();
        let _ = buf.encode(&info_v);

        let a = serde_bencode::to_bytes(&info).unwrap();
        let a = Sha1::digest(&a);
        let a = hex::encode(a);

        let _sh1_digest = Sha1::digest(&buf.outer_buf);
        let hx = hex::encode(_sh1_digest);

        // println!(" gg {:?} {:?}", hx, a);

        // println!("Tracker URL: {}", announce);
        // println!("Length: {}", length);
        println!("Info Hash: {}", hx);
    } else {
        println!("unknown command: {}", args[1])
    };
}
#[test]
fn tt() {
    let encoded_value = "d3:4443:9995:54321d3:3213:3215:54321i99ee3:321l12:000987654321i99eee";
    let mut buf = MyBEncodedBuf::from(encoded_value);
    let decoded_value = decode_bencoded_value(&mut buf);
    let ec = buf.encode(&decoded_value).unwrap();
    let mut buf2 = MyBEncodedBuf {
        outer_buf: vec![],
        pos: 0,
        inner_buf: buf.outer_buf.clone(),
    };
    let decoded_value2 = decode_bencoded_value(&mut buf2);
    let a = serde_bencode::to_bytes(&decoded_value).unwrap();

    println!(
        "{:?}   \n {:?}\n {:?}",
        a,
        buf.outer_buf,
        buf2.outer_buf,
        // String::from_utf8(buf.outer_buf.clone()).unwrap()
    );
    assert_eq!(
        String::from_utf8(buf2.inner_buf.clone()).unwrap(),
        String::from_utf8(buf.inner_buf.clone()).unwrap()
    )
}
