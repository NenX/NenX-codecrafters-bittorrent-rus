use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]

pub struct MyExtensionHandshake {
    pub length: u32,
    pub msg_id: u8,
    pub payload: MyExtensionMsg,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MyExtensionMsg {
    pub ext_msg_id: u8,
    pub dic: MyExtensionMsgInnerDic,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct MyExtensionMsgInnerDic {
    pub m: HashMap<String, usize>,
}
impl MyExtensionHandshake {}

#[cfg(test)]
mod a {
    

    use super::*;

    #[test]
    fn tt() {
        let mut hash_map = HashMap::new();
        hash_map.insert("gg".to_string(), 2);

        let m = MyExtensionMsg {
            ext_msg_id: 1,
            dic: MyExtensionMsgInnerDic { m: hash_map },
        };
        let s = serde_bencode::to_string(&m);
        println!("s {:?}", s)
    }
}
