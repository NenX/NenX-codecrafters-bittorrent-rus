use std::{fs, path::Path};

use anyhow::Context;
use serde::{de::Visitor, Deserialize, Serialize};
use sha1::{Digest, Sha1};



#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyTorrent {
    pub announce: String,
    // #[serde(rename = "created by")]
    // pub create_by: String,
    pub info: MyTorrentInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyTorrentInfo {
    pub name: String,
    #[serde(flatten)]
    pub keys: MyTorrentInfoKeys,
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    pub pieces: MyTorrentPieces,
}
impl MyTorrentInfo {
    pub fn hash(&self) -> String {
        let info_encoded = serde_bencode::to_bytes(self).expect("info to bytes");

        let h = Sha1::digest(info_encoded);
        // let h = self.info_hash();
        
        hex::encode(h)
    }
    pub fn info_hash(&self) -> [u8; 20] {
        let info_encoded =
            serde_bencode::to_bytes(self).expect("re-encode info section should be fine");
        let mut hasher = Sha1::new();
        hasher.update(&info_encoded);
        hasher
            .finalize()
            .into()
    }
    pub fn urlencode(&self) -> String {
        let bytes = self.info_hash();
        let mut s = String::with_capacity(bytes.len() * 3);

        bytes.iter().for_each(|b| {
            s.push('%');
            s.push_str(&hex::encode([*b]));
        });

        s
    }
}
#[derive(Debug, Clone)]
pub struct MyTorrentPieces(pub Vec<[u8; 20]>);
impl MyTorrentPieces {
    pub fn print(&self) {
        self.0
            .iter()
            .for_each(|hash| println!("{}", hex::encode(hash)));
    }
}
impl Serialize for MyTorrentPieces {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let single_slice = self.0.concat();
        serializer.serialize_bytes(&single_slice)
    }
}
struct MyTorrentPiecesVisitor;
impl<'de> Visitor<'de> for MyTorrentPiecesVisitor {
    type Value = MyTorrentPieces;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("pieces des")
    }
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() % 20 != 0 {
            return Err(E::custom("can't % 20"));
        }
        let a = v.chunks_exact(20);
        let b: Vec<[u8; 20]> = a.map(|c| c.try_into().unwrap()).collect();
        Ok(MyTorrentPieces(b))
    }
}
impl<'de> Deserialize<'de> for MyTorrentPieces {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(MyTorrentPiecesVisitor)
    }
}
impl MyTorrent {
    pub fn from_file<T: AsRef<Path>>(file: T) -> Self {
        let b = fs::read(file).expect("read file");
        serde_bencode::from_bytes(&b).context("context").expect("?")
    }
    pub fn single_length(&self) -> Option<usize> {
        match &self.info.keys {
            MyTorrentInfoKeys::SingleFile { length } => Some(*length),
            MyTorrentInfoKeys::MultiFile { files } => None,
        }
    }
 
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MyTorrentInfoKeys {
    SingleFile { length: usize },
    MultiFile { files: Vec<MyTorrentInfoFiles> },
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyTorrentInfoFiles {
    pub length: usize,
    pub path: Vec<String>,
}

#[test]
fn torrent_test() {
    let t = MyTorrent::from_file("sample.torrent");
    println!("{:?}", t)
}
