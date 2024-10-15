use std::net::SocketAddrV4;

use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct MyTrackerRequest {
    // pub pubinfo_hash: [u8; 20],
    pub peer_id: String,
    pub port: u16,
    pub uploaded: usize,
    pub downloaded: usize,
    pub left: usize,
    pub compact: u8,
}
#[derive(Debug, Clone, Deserialize)]
pub struct MyTrackerResponse {
    // pub pubinfo_hash: [u8; 20],
    // pub interval: isize,
    pub complete: usize,
    pub incomplete: usize,
    #[serde(rename = "min interval")]
    pub min_interval: usize,
    pub peers: MyTrackerPeers,
}
#[derive(Debug, Clone)]
pub struct MyTrackerPeers(pub Vec<SocketAddrV4>);
struct MyTrackerPeersVisitor;

impl<'de> Visitor<'de> for MyTrackerPeersVisitor {
    type Value = MyTrackerPeers;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("6 bytes, the first 4 bytes are a peer's IP address and the last 2 are a peer's port number")
    }
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let len = v.len();
        if len % 6 != 0 {
            return Err(E::custom("bad length"));
        }
        let v = v.chunks(6);
        let v: Vec<_> = v
            .map(|bytes| {
                let mut sb = bytes.chunks(4);
                let sb0: [u8; 4] = sb.next().unwrap().try_into().unwrap();
                let sb1: [u8; 2] = sb.next().unwrap().try_into().unwrap();
                SocketAddrV4::new(sb0.into(), u16::from_be_bytes(sb1))
            })
            .collect();
        let p = MyTrackerPeers(v);
        Ok(p)
    }
}
impl<'de> Deserialize<'de> for MyTrackerPeers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(MyTrackerPeersVisitor)
    }
}
impl Serialize for MyTrackerPeers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v: Vec<_> = self
            .0
            .iter()
            .flat_map(|addr| {
                let ip: [u8; 4] = addr.ip().octets();
                let port: [u8; 2] = addr.port().to_be_bytes();
                let mut v = vec![];
                v.extend_from_slice(&ip);
                v.extend_from_slice(&port);
                v
            })
            .collect();
        serializer.serialize_bytes(&v)
    }
}
impl MyTrackerPeers {
    pub fn print(&self) {
        self.0.iter().for_each(|addr| println!("{}", addr));
    }
}
