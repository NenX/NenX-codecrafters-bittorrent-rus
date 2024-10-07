use anyhow::{bail, Context, Result};
use bittorrent_starter_rust::{
    dict_get, dict_get_as, display_value, torrent_task, value_as_bytes, value_as_dict,
    value_as_int, MyBEncodedBuf,
};
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
    torrent_task().unwrap();
}
#[test]
fn tt() {}

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
