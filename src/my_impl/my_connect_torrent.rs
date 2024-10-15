use std::path::Path;

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::{fs, io::AsyncWriteExt, net::TcpStream};
use tokio_util::codec::Framed;

use crate::{
    my_impl::{MyExtHandshakePayload, MyPeerMsgTag, MyPiecePayload},
    sha1_u8_20,
};

use super::{MyConnect, MyPeerMsg, MyPeerMsgFramed, MyTorrent};

impl MyConnect {
    pub async fn pre_download<'a>(
        &'a mut self,
    ) -> Result<Framed<&'a mut TcpStream, MyPeerMsgFramed>> {
        let socket = &mut self.remote_socket;
        let hs = &self.hs_data.unwrap();
        let mut peer_framed = Framed::new(socket, MyPeerMsgFramed);

        let msg = peer_framed
            .next()
            .await
            .expect("peer next")
            .context("peer next")?;
        assert_eq!(msg.tag, MyPeerMsgTag::Bitfield);

        if hs.has_ext_reserved_bit() {
            peer_framed
                .send(MyPeerMsg::ext_handshake())
                .await
                .context("peer send")?;

            let msg = peer_framed
                .next()
                .await
                .expect("peer next")
                .context("peer next")?;
            assert_eq!(msg.tag, MyPeerMsgTag::Extendsion);
            let msg = MyExtHandshakePayload::from_bytes(&msg.payload).expect("parse ext payload");
            println!("Peer Metadata Extension ID: {}", msg.ut_metadata());
            self.ext_hs_payload = Some(msg);
        } else {
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
        }

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

        let mut conn = Self::connect(torrent).await?;

        let mut all: Vec<u8> = vec![];
        let mut peer_framed = conn.pre_download().await?;

        Self::downlaod_piece_impl(torrent, piece_i, &mut all, &mut peer_framed).await?;

        fs::write(output, all).await.context("write all")?;
        Ok(())
    }
    pub async fn downlaod_all<T: AsRef<Path>>(torrent: &MyTorrent, output: T) -> Result<()> {
        println!("download {:?}", torrent);
        let mut conn = Self::connect(torrent).await?;

        let mut all: Vec<u8> = vec![];
        let mut peer_framed = conn.pre_download().await?;

        for (piece_i, _) in torrent.info.pieces.0.iter().enumerate() {
            Self::downlaod_piece_impl(torrent, piece_i, &mut all, &mut peer_framed).await?;
        }

        fs::write(output, all).await.context("write all")?;
        Ok(())
    }
    pub async fn magnet_all<T: AsRef<Path>>(torrent: &MyTorrent, output: T) -> Result<()> {
        println!("download {:?}", torrent);
        let mut conn = Self::connect(torrent).await?;
        let ext_payload = conn.ext_hs_payload.clone().unwrap();
        let mut peer_framed = conn.pre_download().await?;

        peer_framed
            .send(MyPeerMsg::ext_meta_request(ext_payload.ut_metadata(), 0, 0))
            .await
            .context("peer send")?;

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
