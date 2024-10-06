use serde_json::{self, Number};
use std::env;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    let len = encoded_value.len();
    let c = encoded_value.chars().next().unwrap();
    let a = match c {
        'i' if encoded_value.ends_with('e') => {
            let n_str = &encoded_value[1..len - 1];
            let n = n_str.parse::<i64>().unwrap();
            serde_json::Value::Number(Number::from(n))
        }

        _ if c.is_digit(10) => {
            let colon_index = encoded_value.find(':').unwrap();
            let number_string = &encoded_value[..colon_index];
            let number = number_string.parse::<i64>().unwrap();
            let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
            serde_json::Value::String(string.to_string())
        }

        _ => {
            panic!("Unhandled encoded value: {}", encoded_value)
        }
    };
    a
    // If encoded_value starts with a digit, it's a number
    // if encoded_value.chars().next().unwrap().is_digit(10) {
    //     // Example: "5:hello" -> "hello"
    //     let colon_index = encoded_value.find(':').unwrap();
    //     let number_string = &encoded_value[..colon_index];
    //     let number = number_string.parse::<i64>().unwrap();
    //     let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
    //     return serde_json::Value::String(string.to_string());
    // } else {
    //     panic!("Unhandled encoded value: {}", encoded_value)
    // }
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
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
