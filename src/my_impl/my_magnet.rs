use std::collections::HashMap;

use anyhow::{Context, Result};

use crate::{
    e_msg, info_hash_encode,
    my_impl::{MyTrackerRequest, MyTrackerResponse},
};

use super::MyTrackerPeers;

pub struct MyMagnet {
    pub xt: String,
    pub dn: String,
    pub tr: String,
}
const MAGNET_PROTOCOL: &str = "magnet:?";
impl MyMagnet {
    pub fn info_hash(&self) -> Result<[u8; 20]> {
        let xt = &self.xt;

        let v = hex::decode(xt)?;
        assert_eq!(v.len(), 20);

        Ok(v.try_into().unwrap())
    }
    pub fn from_hashmap(m: HashMap<String, String>) -> Result<Self> {
        let xt = m.get("xt").cloned().unwrap_or_default();

        if xt.len() != 40 {
            return e_msg!(;"mag");
        }

        Ok(Self {
            xt: m.get("xt").cloned().unwrap_or_default(),
            dn: m.get("dn").cloned().unwrap_or_default(),
            tr: m.get("tr").cloned().unwrap_or_default(),
        })
    }
    pub fn from_link(link: &str) -> Result<Self> {
        if !link.starts_with(MAGNET_PROTOCOL) {
            return e_msg!(;"");
        }
        let link = &link[MAGNET_PROTOCOL.len()..];

        let a = link.split('&');
        let m: HashMap<String, String> = a
            .filter_map(|part| {
                if !part.contains('=') {
                    return None;
                }
                let mut split = part.split('=');
                let key = split.next().expect("magnet key").to_string();
                let mut value = split.next().expect("magnet value").to_string();
                if value.starts_with("http") {
                    while let Some(idx) = value.find('%') {
                        let b_str = &value[idx + 1..idx + 3];
                        let b = hex::decode(b_str).expect("hex to u8");
                        let b = String::from_utf8(b).expect("u8 -> string");
                        value = value.replace(&value[idx..idx + 3], &b);
                    }
                }
                if value.starts_with("urn:btih:") {
                    value = value.replace("urn:btih:", "");
                }
                Some((key, value))
            })
            .collect();
        Self::from_hashmap(m)
    }

    pub async fn fetch_peers(&self) -> Result<MyTrackerPeers> {
        let len = 1;

        let request_params = MyTrackerRequest {
            // pubinfo_hash: hx,
            peer_id: String::from("00112233445566778899"),
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left: len,
            compact: 1,
        };
        let request_params = serde_urlencoded::to_string(&request_params).context("url encode")?;

        let request_params = format!(
            "{}?info_hash={}&{}",
            self.tr,
            info_hash_encode(self.info_hash()?),
            request_params
        );
        println!("request_params {}", request_params);
        let res_bytes = reqwest::get(request_params).await?.bytes().await?;
        let res: MyTrackerResponse = serde_bencode::from_bytes(&res_bytes)?;
        res.peers.print();

        Ok(res.peers)
    }
}
