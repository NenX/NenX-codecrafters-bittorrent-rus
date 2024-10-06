use core::str;
use std::{collections::HashMap, error::Error};

use serde_json::{Map, Value};

pub struct MyBEncodedBuf {
    pub pos: usize,
    pub string_buf: Vec<u8>,
}
type MyBEncodedResult<T> = Result<T, Box<dyn Error>>;
impl MyBEncodedBuf {
    pub fn get_current_slice(&self) -> &[u8] {
        let a = &self.string_buf[self.pos..];
        a
    }
    pub fn len_bound(&self) -> usize {
        self.string_buf.len()
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
        let chars = self.string_buf.get(self.pos).cloned();
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
    pub fn parse_str(&mut self) -> MyBEncodedResult<Value> {
        let a = self.split_by(b':')?;
        let aa = String::from_utf8_lossy(a.0).to_string();
        let n = aa.parse::<usize>().expect(&format!("parse_str {}", aa));
        let s: Value = String::from_utf8_lossy(&a.1[1..n + 1]).to_string().into();
        self.step(1 + a.0.len() + n)?;

        Ok(s)
    }
    pub fn parse_integer(&mut self) -> MyBEncodedResult<Value> {
        self.step(1)?;

        let a = self.split_by(b'e')?;
        let aa = String::from_utf8_lossy(a.0).to_string();
        let n = aa.parse::<usize>().expect(&format!("parse_integer {}", aa));

        self.step(a.0.len() + 1)?;

        Ok(n.into())
    }
    pub fn parse_list(&mut self) -> MyBEncodedResult<Value> {
        self.step(1)?;
        let mut v = vec![];
        while let Ok(c) = self.peek() {
            match c {
                b'e' => {
                    self.step(1)?;
                    break;
                }

                _ => {
                    let value = self.parse()?;
                    v.push(value);
                    continue;
                }
            };
        }

        Ok(v.into())
    }
    pub fn parse_dict(&mut self) -> MyBEncodedResult<Value> {
        self.step(1)?;
        let mut v = Map::new();
        while let Ok(c) = self.peek() {
            match c {
                b'e' => {
                    self.step(1)?;
                    break;
                }

                _ => {
                    let entry = self.parse_dict_entry()?;
                    v.insert(entry.0, entry.1);
                }
            };
        }

        Ok(v.into())
    }
    pub fn parse_dict_entry(&mut self) -> MyBEncodedResult<(String, Value)> {
        let key = self.parse_dict_entry_key()?;
        let value = self.parse()?;
        // println!("entry {:?}",(&key,&value));
        Ok((key, value))
    }

    pub fn parse_dict_entry_key(&mut self) -> MyBEncodedResult<String> {
        let c = self.peek()?;
        match c {
            b'0'..=b'9' => {
                let value = self.parse_str()?;
                Ok(value.as_str().unwrap().to_owned())
            }
            _ => panic!(
                "Unhandled entry  key encoded value: {:x} {}",
                self.pos,
                str::from_utf8(&[c]).unwrap_or("x")
            ),
        }
    }
    pub fn parse(&mut self) -> MyBEncodedResult<Value> {
        let c = self.peek()?;
        let a = match c {
            b'i' => {
                let n = self.parse_integer()?;
                n
            }
            b'l' => {
                let n = self.parse_list()?;
                n
            }
            b'd' => {
                let n = self.parse_dict()?;
                n
            }

            b'0'..=b'9' => {
                let n = self.parse_str()?;
                n
            }

            _ => {
                panic!("Unhandled encoded value: {:?} {}", self.string_buf, c)
            }
        };
        Ok(a)
    }
}
impl From<&str> for MyBEncodedBuf {
    fn from(value: &str) -> Self {
        Self {
            pos: 0,
            string_buf: value.into(),
        }
    }
}
impl From<&String> for MyBEncodedBuf {
    fn from(value: &String) -> Self {
        Self {
            pos: 0,
            string_buf: value.as_bytes().to_vec(),
        }
    }
}
