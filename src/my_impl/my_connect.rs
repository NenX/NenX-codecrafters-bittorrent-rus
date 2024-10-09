use std::{
    fmt::Error,
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::MyTorrentResult;

use super::MyTorrent;

#[repr(C)]
#[derive(Debug)]
pub struct MyHandShakeData {
    pub length: u8,
    pub bittorrent: [u8; 19],
    pub reserved: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl MyHandShakeData {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        println!("peer_id {:?} ", &peer_id);

        Self {
            length: 19,
            bittorrent: *b"BitTorrent protocol",
            reserved: [0; 8],
            info_hash,
            peer_id,
        }
    }
}
#[derive(Debug)]
pub struct MyConnect {
    pub torrent: MyTorrent,
    pub local_addr: SocketAddrV4,
    pub remote_socket: TcpStream,
}

impl MyConnect {
    pub async fn handshake(torrent: &str, peer: &str) -> Self {
        let torrent = MyTorrent::from_file(torrent);
        let local_addr = peer.parse::<SocketAddrV4>().expect("parse addr");

        let remote_socket = TcpStream::connect(local_addr).await.expect("connect");

        let mut ins = Self {
            torrent,
            local_addr,
            remote_socket,
        };

        let hash = ins.torrent.info.info_hash();

        let mut hs_data = MyHandShakeData::new(hash, *b"00112233445566778899");
        unsafe {
            let _ = ins.handshake_interact(&mut hs_data).await;
        }
        println!("Peer ID: {}", hex::encode(hs_data.peer_id));

        ins
    }


    async unsafe fn handshake_interact(
        &mut self,
        hs_data: *mut MyHandShakeData,
    ) -> MyTorrentResult<()> {
        let handshake_bytes = hs_data as *mut [u8; std::mem::size_of::<MyHandShakeData>()];
        // Safety: Handshake is a POD with repr(c)
        let handshake_bytes: &mut [u8; std::mem::size_of::<MyHandShakeData>()] =
            unsafe { &mut *handshake_bytes };

        self.remote_socket.write_all(handshake_bytes).await?;
        self.remote_socket.read_exact(handshake_bytes).await?;
        Ok(())
    }
}
