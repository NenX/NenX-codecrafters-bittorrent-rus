use std::{collections::HashMap, fmt::format};

use anyhow::{Error, Result};
use serde_bencode::value::Value;
use sha1::{Digest, Sha1};
// pub type MyTorrentResult<T> = Result<T, Box<dyn Error>>;
pub type MyTorrentResult<T> = Result<T>;
#[macro_export]

macro_rules! e_msg {
    ($x:expr) => {
        anyhow::Error::msg($x)
    };
    (;$x:expr) => {
        Err(anyhow::Error::msg($x))
    };
}

pub fn get_sorted_dict_keys(s: &HashMap<Vec<u8>, Value>) -> Vec<Vec<u8>> {
    let mut keys = vec![];
    if let Some(kes) = s.get(&vec![b'*']).cloned() {
        if let Value::List(l) = kes {
            l.iter().for_each(|k| {
                if let Value::Bytes(b) = k {
                    keys.push(b.clone());
                }
            });
        }
    }

    keys
}

fn display_value_impl(value: &Value) {
    match value {
        Value::Bytes(vec) => print!(r#""{}""#, String::from_utf8_lossy(vec).to_string()),
        Value::Int(i) => print!("{}", i),
        Value::List(vec) => {
            let mut len = vec.len();
            print!("[");
            vec.iter().for_each(|v| {
                len -= 1;
                display_value_impl(&v);
                if len > 0 {
                    print!(",");
                }
            });
            print!("]");
        }
        Value::Dict(hash_map) => {
            print!("{{");
            let mut len = hash_map.len();
            let keys = get_sorted_dict_keys(hash_map);
            if keys.len() > 0 {
                keys.iter().for_each(|k| {
                    if let Some(v) = hash_map.get(k) {
                        len -= 1;

                        display_value_impl(&Value::Bytes(k.to_vec()));
                        print!(":");
                        display_value_impl(v);

                        if len > 1 {
                            print!(",");
                        }
                    }
                });
            } else {
                hash_map.iter().for_each(|v| {
                    len -= 1;
                    let k = v.0;
                    if k.contains(&b'*') {
                        return;
                    }
                    display_value_impl(&Value::Bytes(k.to_vec()));

                    print!(":");
                    display_value_impl(&v.1);
                    if len > 0 {
                        print!(",");
                    }
                });
            }

            print!("}}");
        }
    }
}
pub fn value_as_bytes(v: &Value) -> Option<Vec<u8>> {
    match v {
        Value::Bytes(vec) => Some(vec.clone()),
        _ => None,
    }
}
pub fn value_as_int(v: &Value) -> Option<i64> {
    match v {
        Value::Int(vec) => Some(vec.clone()),
        _ => None,
    }
}
pub fn value_as_list(v: &Value) -> Option<Vec<Value>> {
    match v {
        Value::List(vec) => Some(vec.clone()),
        _ => None,
    }
}
pub fn value_as_dict(v: &Value) -> Option<HashMap<Vec<u8>, Value>> {
    match v {
        Value::Dict(vec) => Some(vec.clone()),
        _ => None,
    }
}
pub fn display_value(v: &Value) {
    display_value_impl(v);
    print!("\n");
}

pub fn dict_get_as<T>(m: &Value, k: &str, t: impl Fn(&Value) -> Option<T>) -> MyTorrentResult<T> {
    let announce_value = dict_get(m, k)?;
    Ok(t(&announce_value).ok_or(e_msg!("dict_get"))?)
}
pub fn dict_get(m: &Value, k: &str) -> MyTorrentResult<Value> {
    let decoded_obj = value_as_dict(&m).ok_or(e_msg!("value_as_dict"))?;
    let announce_value = decoded_obj
        .get(&k.as_bytes().to_vec())
        .ok_or(e_msg!("decoded_obj"))?;
    Ok(announce_value.clone())
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
pub fn calc_target_chunk_length(total_len: usize, chunk_len: usize, n: usize, idx: usize) -> usize {

    if n - 1 == idx {
        let md = total_len % chunk_len;
        if md == 0 {
            chunk_len
        } else {
            md
        }
    } else {
        chunk_len
    }
}
pub fn sha1_u8_20<T: AsRef<[u8]>>(data: T) -> [u8; 20] {
    let out = Sha1::digest(data);
    out.into()
}
