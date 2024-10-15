use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::MyTorrentInfo;
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComType(MyExtMetaDataPayloadDic, MyTorrentInfo);
#[derive(Debug, Clone)]

pub struct MyExtMetaDataPayload {
    pub ext_msg_id: u8,
    pub dic: MyExtMetaDataPayloadDic,
    pub info: Option<MyTorrentInfo>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyExtMetaDataPayloadDic {
    pub msg_type: usize,
    pub piece: usize,
    pub total_size: Option<usize>,
}
impl MyExtMetaDataPayload {
    pub fn new(ext_msg_id: u8, msg_type: usize, piece: usize) -> Self {
        let m = MyExtMetaDataPayload {
            ext_msg_id,
            dic: MyExtMetaDataPayloadDic {
                msg_type,
                piece,
                total_size: None,
            },
            info: None,
        };
        m
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut v = vec![];
        v.push(self.ext_msg_id);
        let a = serde_bencode::to_bytes(&self.dic)?;
        v.extend(a);

        if let Some(info) = &self.info {
            let a = serde_bencode::to_bytes(&info)?;
            v.extend(a);
        }
        Ok(v)
    }
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if 0 == b.len() {
            return None;
        }
        let ext_msg_id = b.get(0).unwrap().clone();
        let bb = &b[1..];
        println!("zz {:?}", String::from_utf8_lossy(bb));
        let dic: ComType = serde_bencode::from_bytes(bb).expect("parse ext dic");
        let a = Self {
            ext_msg_id,
            dic: dic.0,
            info: Some(dic.1),
        };
        Some(a)
    }
}

#[cfg(test)]
mod a {

    use super::*;

    #[test]
    fn tt() {}
}
