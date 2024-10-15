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
    my_impl::{
        MyPeerMsgTag, MyPiecePayload, MyTorrentInfoKeys, MyTrackerRequest, MyTrackerResponse,
    },
    sha1_u8_20, MyTorrentResult,
};

use super::{MyMagnet, MyPeerMsg, MyPeerMsgFramed, MyTorrent, MyTrackerPeers};

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
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20], reserved: [u8; 8]) -> Self {
        Self {
            length: 19,
            bittorrent: *b"BitTorrent protocol",
            reserved,
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
        let mut hs_data = MyHandShakeData::new(info_hash, *b"00112233445566778899", [0; 8]);

        let ins = unsafe { Self::new(peer).await.handshake_interact(&mut hs_data).await };
        println!("Peer ID: {}", hex::encode(hs_data.peer_id));

        ins
    }
    pub async fn magnet_handshake(mag: &MyMagnet) -> Result<Self> {
        let a = mag.fetch_peers().await?;
        let peer = a.0.get(0).unwrap().to_string();
        let info_hash = mag.info_hash()?;

        let mut reserved = [0; 8];
        let item = reserved.get_mut(5).unwrap();
        *item = 16;

        let mut hs_data = MyHandShakeData::new(info_hash, *b"00112233445566778899", reserved);
        println!("hs_data {:?} {:?}", peer, hs_data);
        let ins = unsafe {
            Self::new(&peer)
                .await
                .handshake_interact(&mut hs_data)
                .await
        };
        println!("Peer ID: {}", hex::encode(hs_data.peer_id));

        ins
    }

    async unsafe fn handshake_interact(mut self, hs_data: *mut MyHandShakeData) -> Result<Self> {
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
        Ok(self)
    }

    pub async fn pre_download<'a>(
        socket: &'a mut TcpStream,
        // peer_framed: &mut Framed<&mut TcpStream, MyPeerMsgFramed>,
    ) -> Result<Framed<&'a mut TcpStream, MyPeerMsgFramed>> {
        let mut peer_framed = Framed::new(socket, MyPeerMsgFramed);

        let msg = peer_framed
            .next()
            .await
            .expect("peer next")
            .context("peer next")?;
        assert_eq!(msg.tag, MyPeerMsgTag::Bitfield);

        peer_framed
            .send(MyPeerMsg::interested())
            .await
            .context("peer send")?;

        let msg = peer_framed
            .next()
            .await
            .expect("peer next")
            .context("peer next")?;
        assert_eq!(msg.tag, MyPeerMsgTag::Unchoke);
        Ok(peer_framed)
    }
    pub async fn connect(torrent: &MyTorrent) -> Result<MyConnect> {
        println!("downloadpiece_task");
        let peers = torrent.fetch_peers().await?;
        let first_one = &peers.0.first().unwrap().to_string();
        let c = Self::handshake(torrent, first_one).await?;

        Ok(c)
    }
    pub async fn downlaod_piece_at<T: AsRef<Path>>(
        torrent: &MyTorrent,
        output: T,
        piece_i: usize,
    ) -> Result<()> {
        println!("download piece {:?}", torrent);

        let mut c = Self::connect(torrent).await?;
        let peer = &mut c.remote_socket;

        let mut all: Vec<u8> = vec![];
        let mut peer_framed = Self::pre_download(peer).await?;

        Self::downlaod_piece_impl(torrent, piece_i, &mut all, &mut peer_framed).await?;

        fs::write(output, all).await.context("write all")?;
        Ok(())
    }
    pub async fn downlaod_all<T: AsRef<Path>>(torrent: &MyTorrent, output: T) -> Result<()> {
        println!("download {:?}", torrent);
        let mut c = Self::connect(torrent).await?;
        let peer = &mut c.remote_socket;

        let mut all: Vec<u8> = vec![];
        let mut peer_framed = Self::pre_download(peer).await?;

        for (piece_i, _) in torrent.info.pieces.0.iter().enumerate() {
            Self::downlaod_piece_impl(torrent, piece_i, &mut all, &mut peer_framed).await?;
        }

        fs::write(output, all).await.context("write all")?;
        Ok(())
    }
    pub async fn downlaod_piece_impl(
        torrent: &MyTorrent,
        piece_i: usize,
        all: &mut Vec<u8>,
        peer_framed: &mut Framed<&mut TcpStream, MyPeerMsgFramed>,
    ) -> Result<()> {
        let mut piece_v = vec![];
        let piece_hash = torrent.info.pieces.0.get(piece_i).unwrap();

        let reqs = MyPeerMsg::request_iter(piece_i, torrent);
        for m in reqs {
            // let m = MyPeerMsg::request(index, begin, length);

            peer_framed.send(m).await.context("send")?;

            let msg = peer_framed
                .next()
                .await
                .expect("req peer next")
                .context("peer next")?;

            assert_eq!(msg.tag, MyPeerMsgTag::Piece);
            assert!(!msg.payload.is_empty());
            let payload = MyPiecePayload::ref_from_bytes(&msg.payload).expect("piece payload");

            piece_v.extend_from_slice(&payload.block);
        }

        println!("request piece --> len {}", piece_v.len());
        let hash = sha1_u8_20(&piece_v);
        assert_eq!(&hash, piece_hash);
        all.extend_from_slice(&piece_v);

        Ok(())
    }
}

#[tokio::test]
async fn test() {}
