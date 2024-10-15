use std::{collections::HashMap, io, str::FromStr};

use anyhow::Result;
use regex::Replacer;
use reqwest::Url;

use crate::e_msg;

pub struct MyMagnet {
    pub urn_btih: String,
    pub dn: String,
    pub tr: String,
}
const MAGNET_PROTOCOL: &str = "magnet:?";
impl MyMagnet {
    pub fn from_hashmap(m: HashMap<String, String>) -> Self {
        Self {
            urn_btih: m.get("urn_btih").cloned().unwrap_or_default(),
            dn: m.get("dn").cloned().unwrap_or_default(),
            tr: m.get("tr").cloned().unwrap_or_default(),
        }
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
                return Some((key, value));
            })
            .collect();
        Ok(Self::from_hashmap(m))
    }
}
