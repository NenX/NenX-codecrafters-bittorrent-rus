use core::str;
use std::{collections::HashMap, error::Error, fmt::Display};

use clap::builder::Str;
use serde_bencode::value::Value;
use serde_json::Map;
use sha1::digest::crypto_common::Key;

pub struct MyBEncodedBuf {
    pub pos: usize,
    pub inner_buf: Vec<u8>,
    pub outer_buf: Vec<u8>,
}
type MyBEncodedResult<T> = Result<T, Box<dyn Error>>;
impl MyBEncodedBuf {
    pub fn get_current_slice(&self) -> &[u8] {
        let a = &self.inner_buf[self.pos..];
        a
    }
    pub fn len_bound(&self) -> usize {
        self.inner_buf.len()
    }
    pub fn read(&mut self) -> MyBEncodedResult<u8> {
        let ret = self.peek()?;
        self.seek(self.pos + 1)?;

        Ok(ret)
    }
    pub fn peek(&self) -> MyBEncodedResult<u8> {
        if self.pos > self.len_bound() {
            return Err("read".into());
        }
        let chars = self.inner_buf.get(self.pos).cloned();
        let ret = chars.ok_or("peek")?;

        Ok(ret)
    }
    pub fn seek(&mut self, p: usize) -> MyBEncodedResult<()> {
        if p > self.len_bound() {
            return Err("seek".into());
        }
        self.pos = p;
        Ok(())
    }
    pub fn step(&mut self, steps: usize) -> MyBEncodedResult<()> {
        let target = self.pos + steps;
        if target > self.len_bound() {
            return Err(format!("pos {} step {}", self.pos, steps).into());
        }
        self.pos = target;
        Ok(())
    }
    pub fn split_by(&self, value: u8) -> MyBEncodedResult<(&[u8], &[u8])> {
        let data = self.get_current_slice();
        let idx = data.iter().position(|a| *a == value).ok_or("position")?;
        let a = data.split_at(idx);
        Ok(a)
    }
    pub fn decode_str(&mut self) -> MyBEncodedResult<Value> {
        let a = self.split_by(b':')?;
        let aa = String::from_utf8_lossy(a.0).to_string();
        let n = aa.parse::<usize>().expect(&format!("parse_str {}", aa));
        let s: Value = (a.1[1..n + 1].to_vec()).into();
        self.step(1 + a.0.len() + n)?;

        Ok(s)
    }
    pub fn encode_str(&mut self, s: &Vec<u8>) -> MyBEncodedResult<()> {
        let len: String = s.len().to_string();
        // println!(
        //     "ee_str {:?} {:?}",
        //     String::from_utf8(s.to_vec()),
        //     len.as_bytes()
        // );
        self.outer_buf.extend_from_slice(len.as_bytes());
        self.outer_buf.push(b':');
        self.outer_buf.extend_from_slice(s);

        Ok(())
    }
    pub fn decode_integer(&mut self) -> MyBEncodedResult<Value> {
        self.step(1)?;

        let pair = self.split_by(b'e')?;
        let bb = String::from_utf8_lossy(pair.1).to_string();
        let aa = String::from_utf8_lossy(pair.0).to_string();
        let n = aa.parse::<i64>().expect(&format!(
            "parse_integer [{}] [{}] [{}]",
            aa,
            bb,
            String::from_utf8_lossy(&self.inner_buf)
        ));

        self.step(pair.0.len() + 1)?;

        Ok(n.into())
    }
    pub fn encode_integer(&mut self, s: i64) -> MyBEncodedResult<()> {
        self.outer_buf.push(b'i');

        self.outer_buf.extend_from_slice(s.to_string().as_bytes());
        self.outer_buf.push(b'e');

        Ok(())
    }
    pub fn decode_list(&mut self) -> MyBEncodedResult<Value> {
        self.step(1)?;
        let mut v = vec![];
        while let Ok(c) = self.peek() {
            match c {
                b'e' => {
                    self.step(1)?;
                    break;
                }

                _ => {
                    let value = self.decode()?;
                    v.push(value);
                    continue;
                }
            };
        }

        Ok(v.into())
    }
    pub fn encode_list(&mut self, s: &Vec<Value>) -> MyBEncodedResult<()> {
        self.outer_buf.push(b'l');

        s.iter().for_each(|item| {
            let _ = self.encode(item);
        });
        self.outer_buf.push(b'e');

        Ok(())
    }
    pub fn decode_dict(&mut self) -> MyBEncodedResult<Value> {
        self.step(1)?;
        let mut v = Vec::new();
        let mut m = HashMap::new();
        while let Ok(c) = self.peek() {
            match c {
                b'e' => {
                    self.step(1)?;
                    m.insert(vec![b'*'], v.into());
                    break;
                }

                _ => {
                    let entry = self.decode_dict_entry()?;
                    // println!("decode_dict {}", String::from_utf8_lossy(&entry.0));
                    let k: Value = entry.0.clone().into();
                    v.push(k);
                    m.insert(entry.0, entry.1);
                }
            };
        }

        Ok(m.into())
    }
    pub fn encode_dict(&mut self, s: &HashMap<Vec<u8>, Value>) -> MyBEncodedResult<()> {
        self.outer_buf.push(b'd');
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
        if keys.len() > 0 {
            keys.iter().for_each(|k| {
                let v = s.get(k);
                if let Some(value) = v {
                    let _ = self.encode_str(k);
                    let _ = self.encode(value);
                    // println!("encode_dict {}", String::from_utf8_lossy(k));
                }
            });
        } else {
            s.iter().for_each(|item| {
                let _ = self.encode_str(&item.0);
                // println!("encode_dict {}", String::from_utf8_lossy(item.0));
                let _ = self.encode(item.1);
            });
        }

        self.outer_buf.push(b'e');
        Ok(())
    }
    pub fn decode_dict_entry(&mut self) -> MyBEncodedResult<(Vec<u8>, Value)> {
        let key = self.decode_dict_entry_key()?;
        let value = self.decode()?;
        // println!("entry {:?}",(&key,&value));
        Ok((key, value))
    }

    pub fn decode_dict_entry_key(&mut self) -> MyBEncodedResult<Vec<u8>> {
        let c = self.peek()?;
        match c {
            b'0'..=b'9' => {
                let value = self.decode_str()?;
                match &value {
                    Value::Bytes(vec) => Ok(vec.clone()),

                    _ => Err("key".into()),
                }
            }
            _ => panic!(
                "Unhandled entry  key encoded value: {:x} {}",
                self.pos,
                str::from_utf8(&[c]).unwrap_or("x")
            ),
        }
    }
    pub fn decode(&mut self) -> MyBEncodedResult<Value> {
        let c = self.peek()?;
        let a = match c {
            b'i' => {
                let n = self.decode_integer()?;
                n
            }
            b'l' => {
                let n = self.decode_list()?;
                n
            }
            b'd' => {
                let n = self.decode_dict()?;
                n
            }

            b'0'..=b'9' => {
                let n = self.decode_str()?;
                n
            }

            _ => {
                panic!("Unhandled encoded value: {:?} {}", self.inner_buf, c)
            }
        };
        Ok(a)
    }
    pub fn encode(&mut self, value: &Value) -> MyBEncodedResult<()> {
        let a = match value {
            Value::Int(number) => self.encode_integer(*number),
            Value::Bytes(s) => self.encode_str(s),
            Value::List(vec) => self.encode_list(&vec),
            Value::Dict(map) => self.encode_dict(&map),
        };
        a
    }
    pub fn display_value_impl(&self, value: &Value, level: usize) {
        match value {
            Value::Bytes(vec) => print!("'{}'", String::from_utf8_lossy(vec).to_string()),
            Value::Int(i) => print!("{}", i),
            Value::List(vec) => {
                print!("List[");
                vec.iter().for_each(|v| {
                    self.display_value_impl(&v, level + 1);
                    print!(",");
                });
                print!("]");
            }
            Value::Dict(hash_map) => {
                print!("Dict{{\n");
                hash_map.iter().for_each(|v| {
                    print!("{}{}", "  ".repeat(level), String::from_utf8_lossy(v.0));
                    print!(": ");
                    self.display_value_impl(&v.1, level + 1);

                    print!(",\n");
                });
                print!("{}}}", "  ".repeat(level - 1),);
            }
        }
    }
    pub fn value_as_bytes(&self, value: &Value) -> Option<Vec<u8>> {
        match value {
            Value::Bytes(vec) => Some(vec.clone()),
            _ => None,
        }
    }
    pub fn value_as_int(&self, value: &Value) -> Option<i64> {
        match value {
            Value::Int(vec) => Some(vec.clone()),
            _ => None,
        }
    }
    pub fn value_as_list(&self, value: &Value) -> Option<Vec<Value>> {
        match value {
            Value::List(vec) => Some(vec.clone()),
            _ => None,
        }
    }
    pub fn value_as_dict(&self, value: &Value) -> Option<HashMap<Vec<u8>, Value>> {
        match value {
            Value::Dict(vec) => Some(vec.clone()),
            _ => None,
        }
    }
    pub fn display_value(&self, value: &Value) {
        self.display_value_impl(value, 1);
        print!("\n");
    }
}
impl From<&str> for MyBEncodedBuf {
    fn from(value: &str) -> Self {
        Self {
            pos: 0,
            inner_buf: value.into(),
            outer_buf: vec![],
        }
    }
}
impl From<&String> for MyBEncodedBuf {
    fn from(value: &String) -> Self {
        Self {
            pos: 0,
            inner_buf: value.as_bytes().to_vec(),
            outer_buf: vec![],
        }
    }
}
impl From<Vec<u8>> for MyBEncodedBuf {
    fn from(value: Vec<u8>) -> Self {
        Self {
            pos: 0,
            inner_buf: value,
            outer_buf: vec![],
        }
    }
}
impl From<&Vec<u8>> for MyBEncodedBuf {
    fn from(value: &Vec<u8>) -> Self {
        Self {
            pos: 0,
            inner_buf: value.clone(),
            outer_buf: vec![],
        }
    }
}

#[test]
fn tt() {
    let s: Vec<u8> = vec![
        0x70, 0x69, 0x65, 0x63, 0x65, 0x20, 0x6c, 0x65, 0x6e, 0x67, 0x74, 0x68,
    ];
    let len = s.len().to_string();
    println!("{:x?}", len.as_bytes());
    println!("{:x?}", s);
}
#[test]
fn aa() {
    let mut m = HashMap::new();

    m.insert("b", 1);
    m.insert("a", 1);
    m.insert("c", 1);
    m.insert("d", 1);

    m.iter().for_each(|i| println!("ii {}\n", i.0));
    for i in m {
        println!("ii {}", i.0)
    }
}
