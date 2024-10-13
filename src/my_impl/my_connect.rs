use std::{net::SocketAddrV4, path::Path, slice};

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::{
    fs, io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream
};
use tokio_util::codec::Framed;

use crate::{my_impl::MyPeerMsgTag, peers_task, sha1_u8_20, MyTorrentResult};

use super::{MyPeerMsg, MyPeerMsgFramed, MyTorrent};

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
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        let bytes = self as *mut Self as *mut [u8; std::mem::size_of::<Self>()];
        // Safety: Handshake is a POD with repr(c)
        let bytes: &mut [u8; std::mem::size_of::<Self>()] = unsafe { &mut *bytes };
        bytes
    }
}
#[derive(Debug)]
pub struct MyConnect {
    pub torrent: MyTorrent,
    pub local_addr: SocketAddrV4,
    pub remote_socket: TcpStream,
}

impl MyConnect {
    pub async fn handshake<T: AsRef<Path>>(torrent: T, peer: &str) -> Self {
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
    pub async fn downlaod_piece_at<T: AsRef<Path>>(
        torrent: T,
        output: T,
        piece_i: usize,
    ) -> Result<()> {
        println!("downloadpiece_task");
        let t = MyTorrent::from_file(&torrent);
        let peers = peers_task(&torrent).await?;
        let first_one = &peers.0.get(0).unwrap().to_string();
        let mut c = Self::handshake(torrent, first_one).await;
        let peer = &mut c.remote_socket;

        let length = t.single_length().unwrap();
        let piece_hash = t.info.pieces.0.get(piece_i).unwrap();

        let mut all = Vec::with_capacity(length);
        let reqs = MyPeerMsg::request_iter(piece_i, &t);
        let mut peer_framed = Framed::new(peer, MyPeerMsgFramed);

        let msg = peer_framed.next().await.expect("peer next").context("peer next")?;
        assert_eq!(msg.tag, MyPeerMsgTag::Bitfield);
  

        let msg = peer_framed
            .send(MyPeerMsg::interested())
            .await
            .context("peer send")?;

        let msg = peer_framed.next().await.expect("peer next").context("peer next")?;
        println!("interested back {:?}", msg);

        for m in reqs {
            peer_framed.send(m).await.context("send")?;

            let msg = peer_framed.next().await.expect("peer next").context("peer next")?;
            all.extend_from_slice(&msg.payload);
            println!("x {:?}", msg);
        }
        println!("len {}", all.len());

        let hash = sha1_u8_20(&all);
        println!("piece_hash {:x?}",hash);
        println!("piece_hash {:x?}",piece_hash);
        assert_eq!(&hash, piece_hash);

        fs::write(output, all).await.context("write all")?;
        Ok(())
    }
}

#[repr(C)]
pub struct MyRequest {
    index: [u8; 4],
    begin: [u8; 4],
    length: [u8; 4],
}
impl MyRequest {
    pub fn new(index: u32, begin: u32, length: u32) -> Self {
        Self {
            index: index.to_be_bytes(),
            begin: begin.to_be_bytes(),
            length: length.to_be_bytes(),
        }
    }
    pub fn index(&self) -> u32 {
        u32::from_be_bytes(self.index)
    }
    pub fn begin(&self) -> u32 {
        u32::from_be_bytes(self.begin)
    }
    pub fn length(&self) -> u32 {
        u32::from_be_bytes(self.length)
    }
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        let bytes = self as *mut Self as *mut [u8; std::mem::size_of::<Self>()];
        // Safety: Handshake is a POD with repr(c)
        let bytes: &mut [u8; std::mem::size_of::<Self>()] = unsafe { &mut *bytes };
        bytes
    }
    pub fn as_bytes(&mut self) -> &[u8] {
        let bytes = self as *const Self as *const [u8; std::mem::size_of::<Self>()];
        // Safety: Handshake is a POD with repr(c)
        let bytes = unsafe { &*bytes };
        bytes
    }
}
#[derive(Debug)]
#[repr(C)]
pub struct MyPiece<T: ?Sized = [u8]> {
    index: [u8; 4],
    begin: [u8; 4],
    x: [u8; 4],
    block: T,
}
impl MyPiece {
    pub fn index(&self) -> u32 {
        u32::from_be_bytes(self.index)
    }
    pub fn begin(&self) -> u32 {
        u32::from_be_bytes(self.begin)
    }
    pub fn block(&self) -> &[u8] {
        &self.block
    }
    const PIECE_LEAD: usize = std::mem::size_of::<MyPiece<()>>();
    pub fn ref_from_bytes(data: &[u8]) -> Option<&Self> {
        if data.len() < Self::PIECE_LEAD {
            return None;
        }
        let n = data.len();
        // NOTE: The slicing here looks really weird. The reason we do it is because we need the
        // length part of the fat pointer to Piece to old the length of _just_ the `block` field.
        // And the only way we can change the length of the fat pointer to Piece is by changing the
        // length of the fat pointer to the slice, which we do by slicing it. We can't slice it at
        // the front (as it would invalidate the ptr part of the fat pointer), so we slice it at
        // the back!
        let piece = &data[..n - Self::PIECE_LEAD] as *const [u8] as *const MyPiece;
        // Safety: Piece is a POD with repr(c), _and_ the fat pointer data length is the length of
        // the trailing DST field (thanks to the PIECE_LEAD offset).
        Some(unsafe { &*piece })
    }
}
#[tokio::test]
async fn test() {
    let a = MyConnect::downlaod_piece_at("sample.torrent", "./a", 0)
        .await
        .expect("msg");
}
