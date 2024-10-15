use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug)]

pub struct MyExtHandshakePayload {
    pub ext_msg_id: u8,
    pub dic: MyExtHandshakeDic,
}
impl MyExtHandshakePayload {
    pub fn new(ext_msg_id: u8, dic: HashMap<String, usize>) -> Self {
        let m = MyExtHandshakePayload {
            ext_msg_id,
            dic: dic.into(),
        };
        m
    }
    pub fn ut_metadata(&self) -> u8 {
        *self.dic.m.get("ut_metadata").expect("get ut_metadata") as u8
    }
    pub fn default() -> Self {
        let mut dic: HashMap<String, usize> = HashMap::new();
        dic.insert("ut_metadata".into(), 2);
        let m = MyExtHandshakePayload {
            ext_msg_id: 0,
            dic: dic.into(),
        };
        m
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut v = vec![];
        v.push(self.ext_msg_id);
        let a = serde_bencode::to_bytes(&self.dic)?;
        v.extend(a);
        Ok(v)
    }
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if 0 == b.len() {
            return None;
        }
        let ext_msg_id = b.get(0).unwrap().clone();
        let dic: MyExtHandshakeDic = serde_bencode::from_bytes(&b[1..]).expect("parse ext dic");
        let a = Self { ext_msg_id, dic };
        Some(a)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MyExtHandshakeDic {
    pub m: HashMap<String, usize>,
}
impl From<HashMap<String, usize>> for MyExtHandshakeDic {
    fn from(m: HashMap<String, usize>) -> Self {
        Self { m }
    }
}

#[cfg(test)]
mod a {

    use super::*;

    #[test]
    fn tt() {
        let mut hash_map = HashMap::new();
        hash_map.insert("gg".to_string(), 2);

        let m = MyExtHandshakeDic { m: hash_map };
        let s = serde_bencode::to_string(&m);
        println!("s {:?}", s)
    }
}
