use std::collections::HashMap;

use anyhow::Result;

#[derive(Debug, Clone)]

pub struct MyExtMetaDataPayload {
    pub ext_msg_id: u8,
    pub dic: HashMap<String, usize>,
}
impl MyExtMetaDataPayload {
    pub fn new(ext_msg_id: u8, dic: HashMap<String, usize>) -> Self {
        let m = MyExtMetaDataPayload { ext_msg_id, dic };
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
        let dic: HashMap<String, usize> =
            serde_bencode::from_bytes(&b[1..]).expect("parse ext dic");
        let a = Self { ext_msg_id, dic };
        Some(a)
    }
}

#[cfg(test)]
mod a {

    use super::*;

    #[test]
    fn tt() {}
}
