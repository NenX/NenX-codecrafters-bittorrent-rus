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

use super::{MyPeerMsg, MyPeerMsgFramed, MyTorrent, MyTrackerPeers};

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
    pub async fn handshake(torrent: &MyTorrent, peer: &str) -> Self {
        let local_addr = peer.parse::<SocketAddrV4>().expect("parse addr");

        let remote_socket = TcpStream::connect(local_addr).await.expect("connect");

        let mut ins = Self {
            torrent: torrent.clone(),
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
    pub async fn fetch_peers(b: &MyTorrent) -> MyTorrentResult<MyTrackerPeers> {
        let len = if let MyTorrentInfoKeys::SingleFile { length } = b.info.keys {
            length
        } else {
            todo!()
        };

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
            b.announce,
            b.info.urlencode(),
            request_params
        );
        println!("request_params {}", request_params);
        let res_bytes = reqwest::get(request_params).await?.bytes().await?;
        let res: MyTrackerResponse = serde_bencode::from_bytes(&res_bytes)?;
        res.peers.print();

        Ok(res.peers)
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
        let peers = Self::fetch_peers(torrent).await?;
        let first_one = &peers.0.first().unwrap().to_string();
        let c = Self::handshake(torrent, first_one).await;

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
