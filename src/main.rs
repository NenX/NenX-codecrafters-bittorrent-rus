use bittorrent_starter_rust::MyBEncodedBuf;
use serde_json::{self};
use std::{env, error::Error};

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &mut MyBEncodedBuf) -> serde_json::Value {
    let a = encoded_value.parse().unwrap();
    a
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main()  {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];
    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        // println!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let mut buf = MyBEncodedBuf::from(encoded_value);
        let decoded_value = decode_bencoded_value(&mut buf);
        println!("{}", decoded_value.to_string());
    } else if command == "info" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        // println!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let file_name = &args[2];
        let a = std::fs::read(file_name).unwrap();
        let mut buf = MyBEncodedBuf {
            pos: 0,
            string_buf: a,
        };
        let decoded_value = decode_bencoded_value(&mut buf);
        let announce = decoded_value
            .as_object()
            .unwrap()
            .get("announce")
            .unwrap()
            .as_str()
            .unwrap();
        let info = decoded_value
            .as_object()
            .unwrap()
            .get("info")
            .unwrap()
            .as_object()
            .unwrap();
        let length = info.get("length").unwrap().as_i64().unwrap();
        println!("Tracker URL: {:}", announce);
        println!("Length: {:#?}", length);
    } else {
        println!("unknown command: {}", args[1])
    };
}
