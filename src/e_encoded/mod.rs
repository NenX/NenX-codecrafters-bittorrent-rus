use core::str;
use std::{collections::HashMap, error::Error};

use serde_json::{Map, Value};

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
        let s: Value = String::from_utf8_lossy(&a.1[1..n + 1]).to_string().into();
        self.step(1 + a.0.len() + n)?;

        Ok(s)
    }
    pub fn encode_str(&mut self, s: &str) -> MyBEncodedResult<()> {
        let len = s.as_bytes().len().to_string();
        self.outer_buf.extend_from_slice(len.as_bytes());
        self.outer_buf.push(b':');
        self.outer_buf.extend_from_slice(s.as_bytes());

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
        let mut v = Map::new();
        while let Ok(c) = self.peek() {
            match c {
                b'e' => {
                    self.step(1)?;
                    break;
                }

                _ => {
                    let entry = self.decode_dict_entry()?;
                    v.insert(entry.0, entry.1);
                }
            };
        }

        Ok(v.into())
    }
    pub fn encode_dict(&mut self, s: &Map<String, Value>) -> MyBEncodedResult<()> {
        self.outer_buf.push(b'd');

        s.iter().for_each(|item| {
            let _ = self.encode_str(item.0);

            let _ = self.encode(item.1);
        });
        self.outer_buf.push(b'e');

        Ok(())
    }
    pub fn decode_dict_entry(&mut self) -> MyBEncodedResult<(String, Value)> {
        let key = self.decode_dict_entry_key()?;
        let value = self.decode()?;
        // println!("entry {:?}",(&key,&value));
        Ok((key, value))
    }

    pub fn decode_dict_entry_key(&mut self) -> MyBEncodedResult<String> {
        let c = self.peek()?;
        match c {
            b'0'..=b'9' => {
                let value = self.decode_str()?;
                Ok(value.as_str().unwrap().to_owned())
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
            Value::Number(number) => self.encode_integer(number.as_i64().unwrap()),
            Value::String(s) => self.encode_str(s),
            Value::Array(vec) => self.encode_list(&vec),
            Value::Object(map) => self.encode_dict(&map),
            _ => panic!("encode {}", value),
        };
        a
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
