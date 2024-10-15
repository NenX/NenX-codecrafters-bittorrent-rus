use std::{net::SocketAddrV4, path::Path};

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_util::codec::Framed;

use crate::{
    my_impl::{MyHandShakeData, MyPeerMsgTag, MyPiecePayload},
    sha1_u8_20,
};

use super::{MyMagnet, MyPeerMsg, MyPeerMsgFramed, MyTorrent};
#[derive(Debug)]
pub struct MyConnect {
    pub local_addr: SocketAddrV4,
    pub remote_socket: TcpStream,
}

impl MyConnect {
    pub async fn new(peer: &str) -> Self {
        let local_addr = peer.parse::<SocketAddrV4>().expect("parse addr");

        let remote_socket = TcpStream::connect(local_addr).await.expect("connect");

        Self {
            local_addr,
            remote_socket,
        }
    }
    pub async fn handshake(torrent: &MyTorrent, peer: &str) -> Result<Self> {
        let info_hash = torrent.info.info_hash();
        let mut hs_data = MyHandShakeData::new(info_hash, *b"49756936445566778899");
        let mut ins = Self::new(peer).await;
        unsafe {
            ins.handshake_interact(&mut hs_data)
                .await
                .expect("handshake")
        };
        println!("Peer ID: {}", hex::encode(hs_data.peer_id));

        Ok(ins)
    }
    pub async fn magnet_handshake(mag: &MyMagnet) -> Result<Self> {
        let peer = mag.fetch_peers().await?;
        let peer = peer.0.get(0).unwrap().to_string();
        let info_hash = mag.info_hash()?;
        let mut ins = Self::new(&peer).await;

        let mut hs_data = MyHandShakeData::new(info_hash, *b"49756936445566778899");
        hs_data.set_ext_reserved_bit();
        unsafe { ins.handshake_interact(&mut hs_data).await.unwrap() };
        println!("Peer ID: {}", hex::encode(hs_data.peer_id));

        Ok(ins)
    }

    async unsafe fn handshake_interact(&mut self, hs_data: *mut MyHandShakeData) -> Result<()> {
        let handshake_bytes = hs_data as *mut [u8; std::mem::size_of::<MyHandShakeData>()];
        // Safety: Handshake is a POD with repr(c)
        let handshake_bytes: &mut [u8; std::mem::size_of::<MyHandShakeData>()] =
            unsafe { &mut *handshake_bytes };
        let msg1 = "handshake_interact write";
        let msg2 = "handshake_interact read";

        self.remote_socket
            .write_all(handshake_bytes)
            .await
            .context(msg1)
            .expect(msg1);
        self.remote_socket
            .read_exact(handshake_bytes)
            .await
            .context(msg2)
            .expect(msg2);
        (*hs_data).has_ext_reserved_bit();
        Ok(())
    }
}

#[tokio::test]
async fn test() {}
