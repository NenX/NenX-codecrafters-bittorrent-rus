use std::path::Path;

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::{fs, io::AsyncWriteExt, net::TcpStream};
use tokio_util::codec::Framed;

use crate::{
    my_impl::{MyExtHandshakePayload, MyExtMetaDataPayload, MyPeerMsgTag, MyPiecePayload},
    sha1_u8_20,
};

use super::{MyConnect, MyMagnet, MyPeerMsg, MyPeerMsgFramed, MyTorrent, MyTorrentInfo};

impl MyConnect {
    pub async fn pre_download(&mut self) -> Result<Framed<&mut TcpStream, MyPeerMsgFramed>> {
        let socket = &mut self.remote_socket;
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
    pub async fn magnet_pre_download(
        &mut self,
    ) -> Result<(
        Framed<&mut TcpStream, MyPeerMsgFramed>,
        MyExtHandshakePayload,
    )> {
        let socket = &mut self.remote_socket;
        let hs = &self.hs_data.unwrap();
        assert!(hs.has_ext_reserved_bit());

        let mut peer_framed = Framed::new(socket, MyPeerMsgFramed);

        let msg = peer_framed
            .next()
            .await
            .expect("peer next")
            .context("peer next")?;
        assert_eq!(msg.tag, MyPeerMsgTag::Bitfield);

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

        Ok((peer_framed, msg))
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
        let mut peer_framed = conn.pre_download().await?;

        let mut all: Vec<u8> = vec![];

        Self::downlaod_piece_impl(piece_i, &torrent.info, &mut all, &mut peer_framed).await?;

        fs::write(output, all).await.context("write all")?;
        Ok(())
    }
    pub async fn downlaod_all<T: AsRef<Path>>(torrent: &MyTorrent, output: T) -> Result<()> {
        println!("download {:?}", torrent);
        let mut conn = Self::connect(torrent).await?;
        let mut peer_framed = conn.pre_download().await?;
        let mut all: Vec<u8> = vec![];

        for (piece_i, _) in torrent.info.pieces.0.iter().enumerate() {
            Self::downlaod_piece_impl(piece_i, &torrent.info, &mut all, &mut peer_framed).await?;
        }

        fs::write(output, all).await.context("write all")?;
        Ok(())
    }
    pub async fn magnet_extension_handshake(
        peer_framed: &mut Framed<&mut TcpStream, MyPeerMsgFramed>,
        ut_metadata: u8,
    ) -> Result<MyExtMetaDataPayload> {
        peer_framed
            .send(MyPeerMsg::ext_meta_request(ut_metadata, 0, 0))
            .await
            .context("peer send")?;

        let msg = peer_framed
            .next()
            .await
            .expect("peer next")
            .context("peer next")?;
        assert_eq!(msg.tag, MyPeerMsgTag::Extendsion);

        let a = MyExtMetaDataPayload::from_bytes(&msg.payload).expect("parse magnet info");

        a.info.clone().expect("info").print();
        Ok(a)
    }
    pub async fn magnet_info(mag: &MyMagnet) -> Result<()> {
        let mut conn = Self::magnet_handshake(mag).await?;

        let (mut peer_framed, payload) = conn.magnet_pre_download().await?;

        Self::magnet_extension_handshake(&mut peer_framed, payload.ut_metadata()).await?;
        mag.print();

        Ok(())
    }
    pub async fn magnet_downlaod_piece_at(
        mag: &MyMagnet,
        output: impl AsRef<Path>,
        piece_i: usize,
    ) -> Result<()> {
        let mut conn = Self::magnet_handshake(mag).await?;

        let (mut peer_framed, payload) = conn.magnet_pre_download().await?;
        println!("qqq 0");

        let meta =
            Self::magnet_extension_handshake(&mut peer_framed, payload.ut_metadata()).await?;
        mag.print();
        println!("qqq 1");
        let mut all: Vec<u8> = vec![];

        Self::downlaod_piece_impl(piece_i, &meta.info.unwrap(), &mut all, &mut peer_framed).await?;
        println!("qqq 2");

        fs::write(output, all).await.context("write all")?;

        Ok(())
    }

    pub async fn downlaod_piece_impl(
        piece_i: usize,
        info: &MyTorrentInfo,
        all: &mut Vec<u8>,
        peer_framed: &mut Framed<&mut TcpStream, MyPeerMsgFramed>,
    ) -> Result<()> {
        let mut piece_v = vec![];
        let piece_hash = info.pieces.0.get(piece_i).unwrap();

        let reqs = MyPeerMsg::request_iter(piece_i, info);
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
