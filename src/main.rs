use anyhow::{bail, Context, Result};
use bittorrent_starter_rust::MyBEncodedBuf;
use serde_bencode::value::Value;
use serde_json::{self};
use sha1::{Digest, Sha1};
use std::{env, error::Error, fs};

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &mut MyBEncodedBuf) -> Value {
    let a = encoded_value.decode().unwrap();
    a
}

// Usage: 70edcac2611a8829ebf467a6849f5d8408d9d8f4
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
        buf.display_value(&decoded_value);
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
        // buf.display_value(&decoded_value);

        let decoded_obj = buf.value_as_dict(&decoded_value).unwrap();
        let announce_value = decoded_obj.get(&"announce".as_bytes().to_vec()).unwrap();
        let announce_vec = buf.value_as_bytes(announce_value).unwrap();
        let announce = String::from_utf8(announce_vec).unwrap();
        let info_v = decoded_obj.get(&"info".as_bytes().to_vec()).unwrap();
        let info = buf.value_as_dict(&info_v).unwrap();
        let length_v = info.get(&"length".as_bytes().to_vec()).unwrap();
        let length = buf.value_as_int(length_v).unwrap();
        let _ = buf.encode(&info_v);

        let _sh1_digest = Sha1::digest(buf.outer_buf.clone());
        let hx = hex::encode(_sh1_digest);

        println!("Tracker URL: {}", announce);
        println!("Length: {}", length);
        println!("Info Hash: {} ", hx);
    } else {
        println!("unknown command: {}", args[1])
    };
}
#[test]
fn tt() {
    let file_path = "sample.torrent";
    let ab = std::fs::read(file_path).unwrap();

    let encoded_value = "d3:4443:9995:54321d3:3213:3215:54321i99ee3:321l12:000987654321i99eee";
    let mut buf = MyBEncodedBuf::from(&ab);
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
        ab,
        buf.outer_buf,
        buf.inner_buf,
        // String::from_utf8(buf.outer_buf.clone()).unwrap()
    );
    // assert_eq!(
    //     String::from_utf8(buf2.inner_buf.clone()).unwrap(),
    //     String::from_utf8(buf.inner_buf.clone()).unwrap()
    // )
}

#[test]
fn ta() -> Result<()> {
    let file_path = "sample.torrent";
    let info = serde_bencode::from_bytes::<Value>(&fs::read(file_path)?)?;
    println!("info {:?}", info);

    if let Value::Dict(dict) = info {
        let announce = dict.get(b"announce".as_ref()).context("no announce")?;
        let info = dict.get(b"info".as_ref()).context("no info")?;
        println!("info {:?}", info);
        let hash = hex::encode(Sha1::digest(serde_bencode::to_bytes(info)?));
        if let (Value::Bytes(announce), Value::Dict(info)) = (announce, info) {
            println!("Tracker URL: {}", String::from_utf8_lossy(announce));
            let length = info.get(b"length".as_ref()).context("no length")?;
            if let Value::Int(length) = length {
                println!("Length: {length}");
                println!("Info Hash: {hash}");
                Ok(())
            } else {
                bail!("Invalid torrent file")
            }
        } else {
            bail!("Invalid torrent file")
        }
    } else {
        bail!("Invalid torrent file")
    }
}
